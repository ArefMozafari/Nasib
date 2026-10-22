/// Hover help for every control. The Windows app uses the same sentences.
enum Tooltip {
    static let findNow = "Search for a working server and connect to the first one that loads a blocked site."
    static let stopFind = "Stop searching and disconnect."
    static let startSurvey = "Try every server on every protocol and record the results, without stopping at the first success."
    static let stopSurvey = "Stop the survey and reconnect to the last server that worked."
    static let showLog = "Open the list of every connection attempt."
    static let autoReconnect = "Check the tunnel every 3 seconds and a blocked site every minute, and search for a new server when either fails."
    static let launchAtLogin = "Start Nasib in the background when you sign in."
    static let save = "Use these cities and protocols from the next search on."
    static let resetToDefaults = "Restore the default cities and protocol order. Nothing changes until you save."
}
