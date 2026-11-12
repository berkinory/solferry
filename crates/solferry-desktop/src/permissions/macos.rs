use std::ptr::NonNull;
use std::sync::Mutex;

use block2::RcBlock;
use futures::channel::oneshot;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_foundation::{NSBundle, NSError};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNAuthorizationStatus, UNNotificationSettings, UNUserNotificationCenter,
};

use super::Status;

/// The notification center aborts a process that isn't in an app bundle, as in development.
fn center() -> Option<Retained<UNUserNotificationCenter>> {
    NSBundle::mainBundle().bundleIdentifier()?;
    Some(UNUserNotificationCenter::currentNotificationCenter())
}

/// Answers once, from whichever thread the system calls back on.
fn answer() -> (impl Fn(Status) + 'static, oneshot::Receiver<Status>) {
    let (sender, receiver) = oneshot::channel();
    let sender = Mutex::new(Some(sender));
    let send = move |status| {
        if let Some(sender) = sender.lock().ok().and_then(|mut sender| sender.take()) {
            let _ = sender.send(status);
        }
    };
    (send, receiver)
}

pub fn notifications() -> oneshot::Receiver<Status> {
    let (send, receiver) = answer();
    let Some(center) = center() else {
        send(Status::Unknown);
        return receiver;
    };
    let handler = RcBlock::new(move |settings: NonNull<UNNotificationSettings>| {
        #[allow(unsafe_code)]
        // SAFETY: the system passes settings that live for the length of this call.
        let settings = unsafe { settings.as_ref() };
        send(match settings.authorizationStatus() {
            UNAuthorizationStatus::NotDetermined => Status::NotAsked,
            UNAuthorizationStatus::Denied => Status::Off,
            _ => Status::Allowed,
        });
    });
    center.getNotificationSettingsWithCompletionHandler(&handler);
    receiver
}

pub fn ask_notifications() -> oneshot::Receiver<Status> {
    let (send, receiver) = answer();
    let Some(center) = center() else {
        send(Status::Unknown);
        return receiver;
    };
    let handler = RcBlock::new(move |granted: Bool, _: *mut NSError| {
        send(if granted.as_bool() { Status::Allowed } else { Status::Off });
    });
    center.requestAuthorizationWithOptions_completionHandler(
        UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
        &handler,
    );
    receiver
}
