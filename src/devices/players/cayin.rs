use tokio::io::AsyncReadExt;

use crate::devices::{adb, error::DeviceError, registry::SyncGuard};

#[derive(Clone, Copy, Debug, Default)]
pub struct CayinN3UltraScanner;

impl CayinN3UltraScanner {
    pub async fn scan(&self, device: &str) -> Result<(), DeviceError> {
        let _guard = SyncGuard::acquire(device).await;
        trigger_cayin_scan_async(device).await
    }
}

async fn trigger_cayin_scan_async(device: &str) -> Result<(), DeviceError> {
    let mut stream = adb::connect_to_adb().await?;

    let transport_cmd = format!("host:transport:{device}");
    adb::send_adb_cmd(&mut stream, &transport_cmd).await?;

    let script = r#"
input keyevent 224
sleep 0.2

for i in 1 2 3 4 5; do
  dumpsys window | grep mCurrentFocus | grep -q 'N3ultraLauncherMainActivity' && break
  input keyevent 4
  input keyevent 3
  sleep 0.3
done

if ! dumpsys window | grep mCurrentFocus | grep -q 'N3ultraLauncherMainActivity'; then
  am start -a android.intent.action.MAIN -c android.intent.category.HOME >/dev/null 2>&1
  sleep 0.5
  input keyevent 3
  sleep 0.3
fi

input tap 600 857
sleep 0.6
for i in 1 2 3; do
  dumpsys window | grep mCurrentFocus | grep -q 'SettingActivity' && break
  input tap 600 857
  sleep 0.5
done

input tap 360 250
sleep 0.6
for i in 1 2 3; do
  dumpsys window | grep mCurrentFocus | grep -q 'ScanActivity' && break
  input tap 360 250
  sleep 0.5
done

input tap 180 981
sleep 0.8

input keyevent 4
sleep 0.3
input keyevent 4
sleep 0.3
input keyevent 3
"#;

    let shell_req = format!("shell:{}", script.trim());
    adb::send_adb_cmd(&mut stream, &shell_req).await?;

    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response).await;

    Ok(())
}
