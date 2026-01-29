use heli_engine::HeliCore;

#[test]
fn test_june_calibration() {
    // June 16, 2024 at Noon UTC
    let ts_june_16: i64 = 1718539200000;
    let degree = HeliCore::get_orbital_degree(ts_june_16);

    // We expect something around 85.0° - 87.0°
    println!("June 16 Heli-Signature: {}°", degree);
    assert!(degree > 80.0 && degree < 90.0);
}
