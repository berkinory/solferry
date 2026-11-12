use futures::channel::oneshot;
use windows::UI::Notifications::{NotificationSetting, ToastNotificationManager};
use windows::core::HSTRING;

use super::Status;

/// Windows turns notifications off per app in its settings, and never asks first.
pub fn notifications() -> oneshot::Receiver<Status> {
    let setting =
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(crate::APP_ID))
            .and_then(|notifier| notifier.Setting());
    let status = match setting {
        Ok(NotificationSetting::Enabled) => Status::Allowed,
        Ok(_) => Status::Off,
        Err(_) => Status::Unknown,
    };
    super::ready(status)
}
