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
    DiscoverySelectAllBindIps,
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
    /// Window management: reload VM window rows from local SQLite.
    WinMgmtReloadDb,
    /// Window management: empty list headline (same placement as device management).
    WinMgmtNoWindows,
    /// Window management: empty list hint (create on Titan Host).
    WinMgmtEmptyHint,
    /// Center create-window dialog: device selector label.
    CenterWinMgmtDevice,
    /// Center: device combo placeholder until user picks a host.
    CenterWinMgmtDevicePlaceholder,
    CenterWinMgmtErrNoDevice,
    /// Center: no rows in registered device list.
    CenterWinMgmtErrNoDevices,
    CenterWinMgmtDbErr,
    CenterWinMgmtToastCreated,
    /// Center: TCP push of VM window row to the selected host failed or was rejected.
    CenterWinMgmtHostSyncErr,
    /// Center create: same host already has this VM ID or directory.
    CenterWinMgmtErrVmDup,
    /// Center settings: TCP port for Titan Host to pull `vm_window_records` (restart Center to apply).
    CenterVmWindowApiTcpPort,
    CenterVmWindowApiTcpPortHint,
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
    HpLanBindIface,
    /// Host settings: no non-virtual IPv4 found for LAN announce bind.
    HpLanBindIfaceNone,
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
    /// Host settings: VM files root (`{root}/{vm_id}` for each window).
    HpSectionVmStorage,
    HpVmRootDir,
    HpVmRootDirHint,
    /// Host settings card: mTLS pairing window for trusting new Centers.
    HpSectionMtlsPairing,
    /// Host settings: SPKI fingerprint label for the local QUIC certificate.
    HpQuicFingerprintLabel,
    /// Host settings: button to open the mTLS pairing window.
    HpQuicPairingOpenBtn,
    /// Host settings: button to close the mTLS pairing window early.
    HpQuicPairingClose,
    /// Host settings: heading for trusted Centers list.
    HpQuicTrustedCentersHeader,
    /// Host settings: shown when the trust store has no entries.
    HpQuicNoTrustedCenters,
    /// Legacy Host setting (no longer in use): kept for backward i18n compat; will be removed.
    HpCenterVmWindowApiAddr,
    HpCenterVmWindowApiAddrHint,

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
    /// Host create-window: numeric folder segment under configured VM root.
    HpWinMgmtVmId,
    HpWinMgmtVmIdHint,
    HpWinMgmtConfirm,
    HpWinMgmtErrDir,
    /// Host create-window: VM id out of range (100–999999999).
    HpWinMgmtErrVmId,
    /// Host create-window: cannot resolve VM root directory (configure in Settings).
    HpWinMgmtErrVmRoot,
    /// Host create-window: this VM ID is already used (same path as an existing row).
    HpWinMgmtErrVmIdDup,
    /// After create: UDP notify to center succeeded (host list updates after center TCP echo).
    HpWinMgmtSavedNotified,
    /// Host create: beacon JSON could not be built.
    HpWinMgmtSaveErr,
    /// Host window tab: pull rows from Titan Center TCP API.
    HpWinMgmtPullCenter,
    /// Host: set Center VM list API address in Settings before sync.
    HpCenterVmApiMissingAddr,

    /// System tray: restore main window (egui apps).
    TrayShowMainWindow,
    /// System tray: quit the application.
    TrayQuit,

    BtnCancel,

    /// Center: TOFU dialog title (manual host with unknown fingerprint).
    CenterTofuDialogTitle,
    /// Center: TOFU dialog explainer.
    CenterTofuDialogSubtitle,
    /// Center: TOFU dialog "Host" field label.
    CenterTofuHostLabel,
    /// Center: TOFU dialog fingerprint label.
    CenterTofuFingerprintLabel,
    /// Center: TOFU dialog risk warning.
    CenterTofuWarning,
    /// Center: TOFU dialog confirm button.
    CenterTofuConfirm,
    /// Center settings: mTLS / trusted hosts section title.
    CenterSettingsMtlsSection,
    /// Center settings: local fingerprint label.
    CenterSettingsLocalFingerprint,
    /// Center settings: trusted hosts header.
    CenterSettingsTrustedHosts,
    /// Center settings: empty trust list state.
    CenterSettingsNoTrustedHosts,
}
