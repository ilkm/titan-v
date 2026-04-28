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

    VmInventoryTitle,
    ColState,
    /// VM tile second line: "Host · {device label}".
    VmTileHostPrefix,

    WindowPreviewTitle,
    WindowPreviewHint,

    MonitorCardDevices,
    MonitorCardWindows,
    MonitorStatTotal,
    MonitorStatOnline,
    MonitorStatOffline,
    MonitorDevicesScopeHint,
    MonitorWindowsScopeHint,

    SlotGridTitle,
    SlotEmpty,
    NoHost,

    /// Titan Host window title and top chrome.
    HpWinTitle,
    HpTabService,
    HpTabBatch,
    HpLangLabel,
    HpListen,
    HpAnnounce,
    HpPollPort,
    HpRegPort,
    HpPeriodic,
    HpPublicAddr,
    HpLabelOverride,
    HpSaveRestart,
    HpBatchTimeout,
    HpBatchFailFast,
    HpHeadingVmPlans,
    HpHeadingVmGroups,
    HpAddExplicitVm,
    HpAddVmGroup,
    HpName,
    HpDelete,
    HpParentVhdx,
    HpDiffDir,
    HpMemBytes,
    HpGen,
    HpSwitch,
    HpAutoStartAfter,
    HpGpuPath,
    HpDynMac,
    HpNoCkpt,
    HpCpuCount,
    HpPrefix,
    HpCount,
    HpDelGroup,
    HpDryRun,
    HpProvision,
    HpLog,

    /// System tray: restore main window (egui apps).
    TrayShowMainWindow,
    /// System tray: quit the application.
    TrayQuit,

    BtnCancel,
}
