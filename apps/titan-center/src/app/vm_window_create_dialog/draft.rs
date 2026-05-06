use titan_common::{VM_WINDOW_FOLDER_ID_MAX, VM_WINDOW_FOLDER_ID_MIN, VmWindowRecord};

use crate::app::i18n::{Msg, UiLang, t};
use crate::app::persist_data::HostEndpoint;
use crate::app::vm_window_db;

/// Draft for the center **创建窗口** modal.
#[derive(Debug, Clone)]
pub(crate) struct CenterVmWindowCreateForm {
    pub dialog_open: bool,
    pub device_ix: Option<usize>,
    pub cpu_count: u32,
    pub memory_mib: u32,
    pub disk_mib: u32,
    pub vm_id: u32,
    pub inline_err: String,
}

impl CenterVmWindowCreateForm {
    pub(crate) fn with_defaults() -> Self {
        Self {
            dialog_open: false,
            device_ix: None,
            cpu_count: 2,
            memory_mib: 4096,
            disk_mib: 131_072,
            vm_id: VM_WINDOW_FOLDER_ID_MIN,
            inline_err: String::new(),
        }
    }
}

pub(crate) fn clamp_device_ix(form: &mut CenterVmWindowCreateForm, n: usize) {
    if let Some(ix) = form.device_ix
        && ix >= n
    {
        form.device_ix = None;
    }
}

pub(crate) fn vm_window_create_prepare_row(
    endpoints: &[HostEndpoint],
    form: &CenterVmWindowCreateForm,
    nonce: &mut u64,
    lang: UiLang,
) -> Result<VmWindowRecord, &'static str> {
    let dev_ix = resolve_dev_ix(endpoints, form.device_ix, lang)?;
    if !(VM_WINDOW_FOLDER_ID_MIN..=VM_WINDOW_FOLDER_ID_MAX).contains(&form.vm_id) {
        return Err(t(lang, Msg::HpWinMgmtErrVmId));
    }
    let mut ep = endpoints[dev_ix].clone();
    Ok(build_row(&mut ep, form, nonce))
}

pub(crate) fn vm_window_local_persist_create(
    row: &VmWindowRecord,
    lang: UiLang,
) -> Result<(), String> {
    let path = vm_window_db::center_vm_window_db_path();
    match vm_window_db::conflicts_for(
        &path,
        &row.record_id,
        &row.device_id,
        row.vm_id,
        &row.vm_directory,
    ) {
        Ok(true) => return Err(t(lang, Msg::CenterWinMgmtErrVmDup).to_string()),
        Ok(false) => {}
        Err(e) => {
            tracing::warn!(error = %e, "vm_window_db: conflicts_for");
            return Err(t(lang, Msg::CenterWinMgmtHostSyncErr).to_string());
        }
    }
    if let Err(e) = vm_window_db::upsert(&path, row) {
        tracing::warn!(error = %e, "vm_window_db: upsert");
        return Err(t(lang, Msg::CenterWinMgmtHostSyncErr).to_string());
    }
    Ok(())
}

fn resolve_dev_ix(
    endpoints: &[HostEndpoint],
    device_ix: Option<usize>,
    lang: UiLang,
) -> Result<usize, &'static str> {
    if endpoints.is_empty() {
        return Err(t(lang, Msg::CenterWinMgmtErrNoDevices));
    }
    let Some(ix) = device_ix else {
        return Err(t(lang, Msg::CenterWinMgmtErrNoDevice));
    };
    if ix >= endpoints.len() {
        return Err(t(lang, Msg::CenterWinMgmtErrNoDevice));
    }
    Ok(ix)
}

fn build_row(
    ep: &mut HostEndpoint,
    form: &CenterVmWindowCreateForm,
    nonce: &mut u64,
) -> VmWindowRecord {
    *nonce = nonce.wrapping_add(1);
    ep.ensure_device_id();
    let ms = unix_millis_now();
    VmWindowRecord {
        record_id: format!("cw-{ms}-{}", *nonce),
        device_id: ep.device_id.clone(),
        host_control_addr: ep.addr.clone(),
        host_label: ep.label.clone(),
        cpu_count: form.cpu_count,
        memory_mib: form.memory_mib,
        disk_mib: form.disk_mib,
        vm_directory: form.vm_id.to_string(),
        vm_id: form.vm_id,
        remark: String::new(),
        created_at_unix_ms: ms,
    }
}

fn unix_millis_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
