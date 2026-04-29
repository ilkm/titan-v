//! Static UI copy keys for Titan Center and Titan Host.

/// Static UI copy keys.
#[derive(Clone, Copy)]
pub enum Msg {
    BrandTitle,
    SettingsTooltip,
    SettingsLangWindowTitle,
    LangRadioEn,
    LangRadioZh,

    NavConnect,
    NavSettings,
    NavHostsVms,
    NavMonitor,

    DiscoveryTitle,
    DiscoveryUdpBlurb,
    DiscoveryCheckbox,
    DiscoveryBindBlurb,
    DiscoveryBindQuickAdd,
    DiscoveryBindScrollHint,
    DiscoveryRefreshIfaces,
    DiscoveryClearBindIps,
    DiscoveryNoIpv4Ifaces,
    IntervalLabel,
    UdpPortLabel,

    HostCollectTitle,
    HostCollectBlurb,
    HostCollectCheckbox,
    HostCollectIntervalLabel,
    HostCollectPollPortLabel,
    HostCollectRegisterPortLabel,

    /// Device management tab when there are no saved hosts (no section title).
    NoDataShort,
    /// Device management: centered headline when the registered list is empty.
    DeviceMgmtNoRegistered,
    /// Hint under empty device list: connect from Settings to populate.
    DeviceMgmtEmptyHint,
    /// Shown on device card preview until host desktop streaming is wired.
    DeviceMgmtDesktopPreviewNote,
    /// Online device card: hover overlay action on desktop preview.
    DeviceMgmtPreviewConfigure,
    /// Online device card: hover overlay — remove this host from the list (red label in UI).
    DeviceMgmtPreviewDelete,
    /// Floating host JSON draft editor (from preview Configure).
    HostConfigWinTitle,
    HostConfigWinLoadDb,
    HostConfigWinSaveDb,
    HostConfigWinPushHost,
    HostConfigWinClose,
    /// Device card: CPU usage label (value appended in UI).
    DeviceMgmtResCpu,
    /// Device card: memory usage label.
    DeviceMgmtResMem,
    /// Device card: network line; values are `down / up` (compact suffixes, no arrow glyphs).
    DeviceMgmtResNet,
    /// Device card: disk I/O line; values are `read / write` (compact suffixes, no arrow glyphs).
    DeviceMgmtResDiskIo,
    /// Device card: user remark section title.
    DeviceMgmtRemarkTitle,
    /// Device card: empty remark hint (double-click to edit).
    DeviceMgmtRemarkDblclkHint,
    BtnAddHost,
    /// Manual add-host dialog (IP + port).
    AddHostDialogTitle,
    AddHostDialogSubtitle,
    AddHostIpLabel,
    AddHostPortLabel,
    AddHostConfirm,
    AddHostInvalidHint,
    /// Add-host: Hello probe in progress (button disabled).
    AddHostVerifying,
    /// Toast when manual add-host Hello fails (offline / timeout / error).
    AddHostOfflineToast,
    /// Status log after manual add-host succeeds.
    AddHostSavedLog,
    /// Device toolbar: send Hello to the currently selected host.
    BtnHostHello,
    /// Device toolbar: open telemetry stream for the selected host.
    BtnHostTelemetry,

    /// Window management: reload VM window rows from local SQLite.
    WinMgmtReloadDb,
    /// Window management: empty list headline (same placement as device management).
    WinMgmtNoWindows,
    /// Window management: empty list hint (create on Titan Host).
    WinMgmtEmptyHint,
    ColState,
    /// VM tile second line: "Host · {device label}".
    VmTileHostPrefix,

    MonitorCardDevices,
    MonitorCardWindows,
    MonitorStatTotal,
    MonitorStatOnline,
    MonitorStatOffline,
    MonitorDevicesScopeHint,
    MonitorWindowsScopeHint,

    NoHost,

    /// Titan Host window title and top chrome.
    HpWinTitle,
    /// Host sidebar: listen / announce / persist (formerly “Service”).
    HpTabSettings,
    /// Host sidebar: window-management tab (aligned with Titan Center window page).
    HpTabWindowMgmt,
    HpLangLabel,
    HpListen,
    HpAnnounce,
    HpPollPort,
    HpRegPort,
    HpPeriodic,
    HpPublicAddr,
    HpLabelOverride,
    HpSaveRestart,
    /// Host settings card: TCP control plane listen.
    HpSectionControlPlane,
    /// Host settings card: LAN UDP announce / Center registration.
    HpSectionLanAnnounce,
    /// Host settings card: display identity overrides.
    HpSectionIdentity,

    /// Host: open dialog to define a new VM window.
    HpWinMgmtCreateBtn,
    HpWinMgmtDialogTitle,
    HpWinMgmtCpu,
    /// Memory field label: MiB where 1024 MiB ≈ 1 GiB.
    HpWinMgmtMem,
    /// Disk field label: MiB where 1024 MiB ≈ 1 GiB.
    HpWinMgmtDisk,
    HpWinMgmtVmDir,
    HpWinMgmtVmDirHint,
    HpWinMgmtConfirm,
    HpWinMgmtErrDir,
    /// After create: local save + UDP notify to center succeeded.
    HpWinMgmtSavedNotified,
    HpWinMgmtSaveErr,

    /// System tray: restore main window (egui apps).
    TrayShowMainWindow,
    /// System tray: quit the application.
    TrayQuit,

    BtnCancel,
}
