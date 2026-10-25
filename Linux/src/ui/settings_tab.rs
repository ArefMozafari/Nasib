//! The Settings tab: the locations in the order to try, the datacenters to add them from, the ranking
//! switch and the protocol order, saved to the engine's `config.json`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};
use nasib_linux::engine_catalog::Catalog;
use nasib_linux::engine_config::{protocol_rows, save, save_problem, Configuration};
use nasib_linux::layout;
use nasib_linux::location_selection::{filter_entries, filter_regions, LocationSelection};
use nasib_linux::reorder::{drag_payload, dragged_index, ReorderList};
use nasib_linux::text::{self, tooltip};

use super::app_model::AppModel;

/// What the tab edits. Nothing reaches the engine until Save.
struct SettingsState {
    /// What was saved, kept so the cities older versions saved carry through a save untouched.
    saved: Configuration,
    protocols: Vec<(String, bool)>,
    rank_by_success: bool,
    /// `None` until the engine's catalog has been read.
    catalog: Option<Catalog>,
    selection: Option<LocationSelection>,
    order_search: String,
    catalog_search: String,
}

struct SettingsTab {
    model: Rc<AppModel>,
    state: RefCell<SettingsState>,
    order_list: gtk::Box,
    order_scroller: gtk::ScrolledWindow,
    catalog_list: gtk::Box,
    count: gtk::Label,
    rank: gtk::CheckButton,
    protocol_list: gtk::Box,
    message: gtk::Label,
    locations_frame: gtk::Frame,
    loading: gtk::Spinner,
    catalog_problem: gtk::Label,
    /// The catalog's check boxes and region links, updated in place so the list keeps its scroll position.
    catalog_checks: RefCell<Vec<(String, gtk::CheckButton)>>,
    region_links: RefCell<Vec<(gtk::Button, Vec<String>)>>,
    /// True while the tab sets check boxes itself, so their toggled handlers don't act on it.
    updating: Cell<bool>,
}

fn link_button(label: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.add_css_class("flat");
    button.add_css_class("caption");
    button
}

fn caption(label: &str) -> gtk::Label {
    let caption = gtk::Label::new(Some(label));
    caption.add_css_class("caption");
    caption.add_css_class("dim-label");
    caption.set_xalign(0.0);
    caption.set_wrap(true);
    caption
}

fn heading(label: &str) -> gtk::Label {
    let heading = gtk::Label::new(Some(label));
    heading.add_css_class("heading");
    heading.set_xalign(0.0);
    heading.set_hexpand(true);
    heading
}

fn remove_children(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

/// Lets a row be dragged onto another in the same list; `on_move` gets the dragged row's index and the
/// target's. A row from the other list is refused.
fn make_draggable(row: &gtk::Box, list: ReorderList, index: usize, on_move: Rc<dyn Fn(usize, usize)>) {
    let source = gtk::DragSource::new();
    source.set_actions(gdk::DragAction::MOVE);
    let payload = drag_payload(list, index);
    source.connect_prepare(move |_, _, _| Some(gdk::ContentProvider::for_value(&payload.to_value())));
    row.add_controller(source);
    let target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
    target.connect_drop(move |_, value, _, _| {
        let Some(from) = value.get::<String>().ok().and_then(|payload| dragged_index(list, &payload)) else {
            return false;
        };
        let on_move = on_move.clone();
        // The lists are rebuilt after the drop, so the move waits until the drop's handler has returned.
        glib::idle_add_local_once(move || on_move(from, index));
        true
    });
    row.add_controller(target);
}

fn framed(content: &gtk::Box) -> gtk::Frame {
    content.set_margin_top(layout::GROUP_PADDING);
    content.set_margin_bottom(layout::GROUP_PADDING);
    content.set_margin_start(layout::GROUP_PADDING);
    content.set_margin_end(layout::GROUP_PADDING);
    gtk::Frame::builder().child(content).build()
}

fn scrolled(content: &gtk::Box, height: i32) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .child(content)
        .min_content_height(height)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .has_frame(true)
        .build()
}

impl SettingsTab {
    /// Sets the catalog's check boxes and region links from the selection.
    fn update_catalog_checks(&self) {
        let state = self.state.borrow();
        let Some(selection) = state.selection.as_ref() else { return };
        self.updating.set(true);
        for (nickname, check) in self.catalog_checks.borrow().iter() {
            check.set_active(selection.is_selected(nickname));
        }
        for (link, nicknames) in self.region_links.borrow().iter() {
            let nicknames: Vec<&str> = nicknames.iter().map(String::as_str).collect();
            link.set_label(if selection.are_all_selected(&nicknames) { text::DESELECT_ALL } else { text::SELECT_ALL });
        }
        self.updating.set(false);
        let total = state.catalog.as_ref().map_or(0, |catalog| catalog.entries().len());
        self.count.set_text(&text::checked_count(selection.ordered().len(), total));
    }

    fn rebuild_order(self: &Rc<Self>) {
        let scroll = self.order_scroller.vadjustment().value();
        remove_children(&self.order_list);
        let state = self.state.borrow();
        let Some(selection) = state.selection.as_ref() else { return };
        // Dragging is off while the order is filtered, since a filtered row's place is ambiguous.
        let can_drag = state.order_search.trim().is_empty();
        let on_move: Rc<dyn Fn(usize, usize)> = {
            let tab = Rc::downgrade(self);
            Rc::new(move |from, to| {
                let Some(tab) = tab.upgrade() else { return };
                if let Some(selection) = tab.state.borrow_mut().selection.as_mut() {
                    selection.move_row(from, to);
                }
                tab.refresh_locations();
            })
        };
        for (index, entry) in filter_entries(selection.ordered(), &state.order_search).into_iter().enumerate() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
            let check = gtk::CheckButton::with_label(&entry.title());
            check.set_active(true);
            let tab = Rc::downgrade(self);
            check.connect_toggled(move |_| {
                let Some(tab) = tab.upgrade() else { return };
                if let Some(selection) = tab.state.borrow_mut().selection.as_mut() {
                    selection.toggle(&entry.nickname);
                }
                glib::idle_add_local_once(move || tab.refresh_locations());
            });
            row.append(&check);
            if can_drag {
                make_draggable(&row, ReorderList::Locations, index, on_move.clone());
            }
            self.order_list.append(&row);
        }
        let adjustment = self.order_scroller.vadjustment();
        glib::idle_add_local_once(move || adjustment.set_value(scroll));
    }

    fn refresh_locations(self: &Rc<Self>) {
        self.rebuild_order();
        self.update_catalog_checks();
    }

    fn rebuild_catalog(self: &Rc<Self>) {
        remove_children(&self.catalog_list);
        let mut checks = Vec::new();
        let mut links = Vec::new();
        {
            let state = self.state.borrow();
            let Some(catalog) = state.catalog.as_ref() else { return };
            for region in filter_regions(catalog, &state.catalog_search) {
                let nicknames: Vec<String> = region.datacenters.iter().map(|entry| entry.nickname.clone()).collect();
                let header = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
                header.append(&heading(&region.name));
                let link = link_button(text::SELECT_ALL);
                let tab = Rc::downgrade(self);
                let region_nicknames = nicknames.clone();
                link.connect_clicked(move |_| {
                    let Some(tab) = tab.upgrade() else { return };
                    {
                        let mut state = tab.state.borrow_mut();
                        let Some(selection) = state.selection.as_mut() else { return };
                        let names: Vec<&str> = region_nicknames.iter().map(String::as_str).collect();
                        if selection.are_all_selected(&names) {
                            selection.deselect(&names);
                        } else {
                            selection.select(&names);
                        }
                    }
                    tab.refresh_locations();
                });
                header.append(&link);
                self.catalog_list.append(&header);
                links.push((link, nicknames));
                for entry in region.datacenters {
                    let check = gtk::CheckButton::with_label(&entry.title());
                    let tab = Rc::downgrade(self);
                    let nickname = entry.nickname.clone();
                    check.connect_toggled(move |_| {
                        let Some(tab) = tab.upgrade() else { return };
                        if tab.updating.get() {
                            return;
                        }
                        if let Some(selection) = tab.state.borrow_mut().selection.as_mut() {
                            selection.toggle(&nickname);
                        }
                        glib::idle_add_local_once(move || tab.refresh_locations());
                    });
                    self.catalog_list.append(&check);
                    checks.push((entry.nickname, check));
                }
            }
        }
        self.catalog_checks.replace(checks);
        self.region_links.replace(links);
        self.update_catalog_checks();
    }

    fn rebuild_protocols(self: &Rc<Self>) {
        remove_children(&self.protocol_list);
        let on_move: Rc<dyn Fn(usize, usize)> = {
            let tab = Rc::downgrade(self);
            Rc::new(move |from, to| {
                let Some(tab) = tab.upgrade() else { return };
                {
                    let mut state = tab.state.borrow_mut();
                    if from < state.protocols.len() {
                        let row = state.protocols.remove(from);
                        let to = to.min(state.protocols.len());
                        state.protocols.insert(to, row);
                    }
                }
                tab.rebuild_protocols();
            })
        };
        for (index, (name, enabled)) in self.state.borrow().protocols.iter().enumerate() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
            let check = gtk::CheckButton::with_label(name);
            check.set_active(*enabled);
            let tab = Rc::downgrade(self);
            check.connect_toggled(move |check| {
                if let Some(tab) = tab.upgrade() {
                    tab.state.borrow_mut().protocols[index].1 = check.is_active();
                }
            });
            row.append(&check);
            make_draggable(&row, ReorderList::Protocols, index, on_move.clone());
            self.protocol_list.append(&row);
        }
    }

    fn set_all_protocols(self: &Rc<Self>, enabled: bool) {
        for row in self.state.borrow_mut().protocols.iter_mut() {
            row.1 = enabled;
        }
        self.rebuild_protocols();
    }

    /// Shows the loading, unreadable and cities-kept states as the macOS app does.
    fn show_catalog_state(&self) {
        let state = self.state.borrow();
        self.loading.set_visible(state.catalog.is_none());
        self.locations_frame.set_visible(state.selection.is_some());
        let problem = match (&state.catalog, &state.selection) {
            (Some(_), None) => Some(text::CITIES_KEPT),
            (Some(catalog), Some(_)) if catalog.regions.is_empty() => Some(text::UNREADABLE_CATALOG),
            _ => None,
        };
        self.catalog_problem.set_visible(problem.is_some());
        self.catalog_problem.set_text(problem.unwrap_or_default());
    }

    fn save(&self) {
        let mut state = self.state.borrow_mut();
        let enabled: Vec<String> = state.protocols.iter().filter(|(_, enabled)| *enabled).map(|(name, _)| name.clone()).collect();
        if let Some(problem) = save_problem(state.selection.as_ref().map(|selection| selection.ordered().len()), enabled.len()) {
            self.message.set_text(problem);
            return;
        }
        let configuration = Configuration {
            locations: match state.selection.as_ref() {
                Some(selection) => Some(selection.nicknames()),
                None => state.saved.locations.clone(),
            },
            rank_by_success: state.rank_by_success,
            protocol_order: enabled,
            cities: state.saved.cities.clone(),
        };
        match save(&configuration, &self.model.paths.configuration_file()) {
            Ok(()) => {
                state.saved = configuration;
                self.message.set_text(text::SAVED);
            }
            Err(error) => self.message.set_text(&error.to_string()),
        }
    }

    fn reset_to_defaults(self: &Rc<Self>) {
        let defaults = Configuration::default();
        {
            let mut state = self.state.borrow_mut();
            if let Some(selection) = state.selection.as_mut() {
                selection.reset_to_defaults();
            }
            state.rank_by_success = defaults.rank_by_success;
            state.protocols = protocol_rows(&defaults);
        }
        self.rank.set_active(defaults.rank_by_success);
        self.message.set_text("");
        self.refresh_locations();
        self.rebuild_protocols();
    }

    /// Reads the catalog off the main thread. Without it, the saved order stays editable on its own.
    fn load_catalog(self: &Rc<Self>) {
        let tab = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Some(model) = tab.upgrade().map(|tab| tab.model.clone()) else { return };
            let catalog = Catalog::decode(&model.read_catalog().await).unwrap_or_default();
            let Some(tab) = tab.upgrade() else { return };
            {
                let mut state = tab.state.borrow_mut();
                state.selection = LocationSelection::new(&state.saved, &catalog);
                state.catalog = Some(catalog);
            }
            tab.show_catalog_state();
            tab.rebuild_catalog();
            tab.rebuild_order();
        });
    }
}

fn search_entry(placeholder: &str) -> gtk::SearchEntry {
    let entry = gtk::SearchEntry::new();
    entry.set_placeholder_text(Some(placeholder));
    entry
}

fn column(title: &str, links: &[&gtk::Button], search: &gtk::SearchEntry, list: &gtk::ScrolledWindow, footer: &impl IsA<gtk::Widget>) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, layout::ROW_SPACING);
    column.set_hexpand(true);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
    header.append(&heading(title));
    for link in links {
        header.append(*link);
    }
    column.append(&header);
    column.append(search);
    column.append(list);
    column.append(footer);
    column
}

pub fn build(model: &Rc<AppModel>) -> gtk::Box {
    let saved = Configuration::load(&model.paths.configuration_file());
    let rank = gtk::CheckButton::with_label(text::RANK_BY_SUCCESS);
    rank.set_tooltip_text(Some(tooltip::RANK_BY_SUCCESS));
    rank.set_active(saved.rank_by_success);
    let order_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let order_scroller = scrolled(&order_list, layout::LOCATION_LISTS_HEIGHT);
    let catalog_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let catalog_scroller = scrolled(&catalog_list, layout::LOCATION_LISTS_HEIGHT);
    let count = caption("");
    let catalog_problem = caption("");
    catalog_problem.add_css_class("settings-error");
    let message = caption("");

    let locations = gtk::Box::new(gtk::Orientation::Vertical, layout::ROW_SPACING);
    let locations_frame = framed(&locations);
    let tab = Rc::new(SettingsTab {
        model: model.clone(),
        state: RefCell::new(SettingsState {
            protocols: protocol_rows(&saved),
            rank_by_success: saved.rank_by_success,
            saved,
            catalog: None,
            selection: None,
            order_search: String::new(),
            catalog_search: String::new(),
        }),
        order_list,
        order_scroller: order_scroller.clone(),
        catalog_list,
        count: count.clone(),
        rank: rank.clone(),
        protocol_list: gtk::Box::new(gtk::Orientation::Vertical, 0),
        message: message.clone(),
        locations_frame: locations_frame.clone(),
        loading: gtk::Spinner::new(),
        catalog_problem: catalog_problem.clone(),
        catalog_checks: RefCell::new(Vec::new()),
        region_links: RefCell::new(Vec::new()),
        updating: Cell::new(false),
    });

    let order_search = search_entry(text::SEARCH_ORDER);
    let catalog_search = search_entry(text::SEARCH_CATALOG);
    let order_deselect = link_button(text::DESELECT_ALL);
    let catalog_select = link_button(text::SELECT_ALL);
    let catalog_deselect = link_button(text::DESELECT_ALL);
    locations.append(&heading(text::LOCATIONS_HEADING));
    locations.append(&caption(text::LOCATIONS_HINT));
    let columns = gtk::Box::new(gtk::Orientation::Horizontal, layout::SECTION_SPACING);
    columns.set_homogeneous(true);
    columns.append(&column(text::YOUR_ORDER, &[&order_deselect], &order_search, &order_scroller, &rank));
    columns.append(&column(text::ADD_LOCATIONS, &[&catalog_select, &catalog_deselect], &catalog_search, &catalog_scroller, &count));
    locations.append(&columns);

    let protocols = gtk::Box::new(gtk::Orientation::Vertical, layout::ROW_SPACING);
    let protocol_select = link_button(text::SELECT_ALL);
    let protocol_deselect = link_button(text::DESELECT_ALL);
    let protocol_header = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
    protocol_header.append(&heading(text::PROTOCOLS_HEADING));
    protocol_header.append(&protocol_select);
    protocol_header.append(&protocol_deselect);
    protocols.append(&protocol_header);
    protocols.append(&caption(text::PROTOCOLS_HINT));
    protocols.append(&scrolled(&tab.protocol_list, layout::PROTOCOL_LIST_HEIGHT));

    let reset = gtk::Button::with_label(text::RESET_TO_DEFAULTS);
    reset.set_tooltip_text(Some(tooltip::RESET_TO_DEFAULTS));
    reset.set_hexpand(true);
    reset.set_halign(gtk::Align::Start);
    let save_button = gtk::Button::with_label(text::SAVE);
    save_button.set_tooltip_text(Some(tooltip::SAVE));
    save_button.add_css_class("suggested-action");
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
    buttons.append(&reset);
    buttons.append(&save_button);

    let page = gtk::Box::new(gtk::Orientation::Vertical, layout::SECTION_SPACING);
    page.set_margin_top(layout::PADDING);
    page.set_margin_bottom(layout::PADDING);
    page.set_margin_start(layout::PADDING);
    page.set_margin_end(layout::PADDING);
    tab.loading.set_size_request(-1, layout::LOCATION_LISTS_HEIGHT);
    tab.loading.set_spinning(true);
    page.append(&tab.loading);
    page.append(&locations_frame);
    page.append(&catalog_problem);
    page.append(&framed(&protocols));
    page.append(&buttons);
    page.append(&message);

    let weak = Rc::downgrade(&tab);
    order_search.connect_search_changed(move |entry| {
        let Some(tab) = weak.upgrade() else { return };
        tab.state.borrow_mut().order_search = entry.text().to_string();
        tab.rebuild_order();
    });
    let weak = Rc::downgrade(&tab);
    catalog_search.connect_search_changed(move |entry| {
        let Some(tab) = weak.upgrade() else { return };
        tab.state.borrow_mut().catalog_search = entry.text().to_string();
        tab.rebuild_catalog();
    });
    // The column links act on what the search shows.
    let weak = Rc::downgrade(&tab);
    order_deselect.connect_clicked(move |_| {
        let Some(tab) = weak.upgrade() else { return };
        {
            let mut state = tab.state.borrow_mut();
            let search = state.order_search.clone();
            let Some(selection) = state.selection.as_mut() else { return };
            let shown: Vec<String> = filter_entries(selection.ordered(), &search).into_iter().map(|entry| entry.nickname).collect();
            selection.deselect(&shown.iter().map(String::as_str).collect::<Vec<_>>());
        }
        tab.refresh_locations();
    });
    for (link, select) in [(&catalog_select, true), (&catalog_deselect, false)] {
        let weak = Rc::downgrade(&tab);
        link.connect_clicked(move |_| {
            let Some(tab) = weak.upgrade() else { return };
            {
                let mut state = tab.state.borrow_mut();
                let shown: Vec<String> = state
                    .catalog
                    .as_ref()
                    .map(|catalog| filter_regions(catalog, &state.catalog_search))
                    .unwrap_or_default()
                    .into_iter()
                    .flat_map(|region| region.datacenters)
                    .map(|entry| entry.nickname)
                    .collect();
                let Some(selection) = state.selection.as_mut() else { return };
                let shown: Vec<&str> = shown.iter().map(String::as_str).collect();
                if select {
                    selection.select(&shown);
                } else {
                    selection.deselect(&shown);
                }
            }
            tab.refresh_locations();
        });
    }
    for (link, enabled) in [(&protocol_select, true), (&protocol_deselect, false)] {
        let weak = Rc::downgrade(&tab);
        link.connect_clicked(move |_| {
            if let Some(tab) = weak.upgrade() {
                tab.set_all_protocols(enabled);
            }
        });
    }
    let weak = Rc::downgrade(&tab);
    rank.connect_toggled(move |rank| {
        if let Some(tab) = weak.upgrade() {
            tab.state.borrow_mut().rank_by_success = rank.is_active();
        }
    });
    let weak = Rc::downgrade(&tab);
    reset.connect_clicked(move |_| {
        if let Some(tab) = weak.upgrade() {
            tab.reset_to_defaults();
        }
    });
    let weak = Rc::downgrade(&tab);
    save_button.connect_clicked(move |_| {
        if let Some(tab) = weak.upgrade() {
            tab.save();
        }
    });

    tab.rebuild_protocols();
    tab.show_catalog_state();
    tab.load_catalog();
    // Every handler holds the tab weakly, so the page keeps it alive until the page goes away.
    page.connect_destroy(move |_| {
        let _held = &tab;
    });
    page
}
