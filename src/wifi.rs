use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::{delay::FreeRtos, modem::Modem};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
    wifi::EspWifi,
};

pub fn connect_wifi_and_update_system_time(modem: &mut Modem) {
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(modem, sys_loop, Some(nvs)).unwrap();

    let mut ssid_heapless = heapless::String::<32>::new();
    ssid_heapless
        .push_str("my_wifi_ssid")
        .expect("SSID exceeds heapless String capacity");

    let mut passwd_heapless = heapless::String::<64>::new();
    passwd_heapless
        .push_str("xxxxxx")
        .expect("PASSWD exceeds heapless String capacity");

    log::info!("Starting Wifi...");

    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: ssid_heapless,
            password: passwd_heapless,
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        log::info!("Waiting for station {:?}", config);
        FreeRtos::delay_ms(2000);
    }
    log::info!("Should be connected now");
    log::info!(
        "IP info: {:?}",
        wifi_driver.sta_netif().get_ip_info().unwrap()
    );

    let ntp = EspSntp::new_default().unwrap();

    log::info!("Synchronizing with NTP Server...");
    while ntp.get_sync_status() != SyncStatus::Completed {
        FreeRtos::delay_ms(1000);
    }
    log::info!("Time Sync Completed");

    wifi_driver.stop().unwrap();
    log::info!("Wifi stopped");
}
