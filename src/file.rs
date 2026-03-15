use std::fs::DirEntry;
use std::path::Path;

use chrono::DateTime;

use crate::alarm::Alarm;
use crate::alarm_heap::AlarmHeap;
use crate::config;

// the folder structure is ./<channel id>/<message id>
// the file structure of <message id> is two lines:
// <unix timestamp>
// <message>

pub fn load() -> AlarmHeap {
    let mut alarm_heap = AlarmHeap::default();

    let top_folder = Path::new(config::WHERE_TO_SAVE);
    if std::fs::exists(top_folder).ok().is_none_or(|exists| !exists) {
        return alarm_heap;
    }

    let channel_dirs = std::fs::read_dir(top_folder)
        .expect("failed to read directory of saved alarms")
        .filter_map(|file: std::io::Result<DirEntry>| -> Option<DirEntry> {
            let Ok(file) = file else {
                return None;
            };
            let Ok(ftype) = file.file_type() else {
                return None;
            };
            if ftype.is_dir() {
                Some(file)
            } else {
                None
            }
        });
    for channel_dir in channel_dirs {
        let this_channels_alarms = std::fs::read_dir(channel_dir.path())
            .expect("failed to load alarms for channel")
            .filter_map(|file: std::io::Result<DirEntry>| -> Option<DirEntry> {
                let Ok(file) = file else {
                    return None;
                };
                let Ok(ftype) = file.file_type() else {
                    return None;
                };
                if ftype.is_file() {
                    Some(file)
                } else {
                    None
                }
            });
        for alarm_file in this_channels_alarms {
            let Ok(file_bytes) = std::fs::read(alarm_file.path()) else {
                continue;
            };
            let Ok(file_text) = String::from_utf8(file_bytes) else {
                continue;
            };
            let Some((timestamp_text, what)) = file_text.split_once("\n") else {
                continue;
            };
            let Ok(unix_seconds) = timestamp_text.parse() else {
                continue;
            };
            let Some(datetime) = DateTime::from_timestamp(unix_seconds, 0) else {
                continue;
            };
            let when = datetime.naive_utc();
            let Ok(message_id) = alarm_file.file_name().into_string() else {
                continue;
            };
            let Ok(channel_id) = channel_dir.file_name().into_string() else {
                continue;
            };
            let what = what.to_string();
            alarm_heap.push(Alarm {
                when,
                what,
                channel_id,
                message_id
            });
        }
    }

    return alarm_heap;
}

pub fn save(alarm: &Alarm) {
    let top_folder = Path::new(config::WHERE_TO_SAVE);
    if std::fs::exists(&top_folder).ok().is_none_or(|exists| !exists) {
        std::fs::create_dir_all(&top_folder).expect("failed to create folder to save alarms to");
    }
    let channel_dir = top_folder.join(&alarm.channel_id);
    if std::fs::exists(&channel_dir).ok().is_none_or(|exists| !exists) {
        std::fs::create_dir(&channel_dir).expect("failed to create folder to save channel alarms to");
    }
    let message_file = channel_dir.join(&alarm.message_id);
    let timestamp = alarm.when.and_utc().timestamp().to_string();
    let file_contents = format!("{}\n{}", timestamp, alarm.what);
    std::fs::write(message_file, file_contents).expect("failed to write alarm file");
}

pub fn delete(alarm: &Alarm) {
    let top_folder = Path::new(config::WHERE_TO_SAVE);
    if std::fs::exists(&top_folder).ok().is_none_or(|exists| !exists) {
        return;
    }
    let channel_dir = top_folder.join(&alarm.channel_id);
    if std::fs::exists(&channel_dir).ok().is_none_or(|exists| !exists) {
        return;
    }
    let message_file = channel_dir.join(&alarm.message_id);
    std::fs::remove_file(message_file).expect("failed to delete alarm file");
}
