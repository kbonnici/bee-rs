use super::*;

#[test]
fn test_rounding_four_thousanths() {
    const TESTED_VALUE: f64 = 0.004;

    let rounded = round_to_hundredth(TESTED_VALUE);

    assert_eq!(rounded, 0.00)
}

#[test]
fn test_rounding_five_thousanths() {
    const TESTED_VALUE: f64 = 0.005;

    let rounded = round_to_hundredth(TESTED_VALUE);

    assert_eq!(rounded, 0.01)
}

#[test]
fn test_rounding_six_thousanths() {
    const TESTED_VALUE: f64 = 0.006;

    let rounded = round_to_hundredth(TESTED_VALUE);

    assert_eq!(rounded, 0.01)
}

#[test]
fn test_new_builder_no_gst() {
    let args = Args {
        pay_rate: 0.0,
        gst: None,
        file: std::path::PathBuf::default(),
    };

    let builder = InvoiceBuilder::new(&args);

    let expected = InvoiceBuilder {
        project_hours_logged: HashMap::new(),
        pay_rate: 0.0,
        gst_rate: 0.0,
    };
    assert_eq!(builder, expected);
}

#[test]
fn test_new_builder_with_gst() {
    let args = Args {
        pay_rate: 0.0,
        gst: Some(10.0),
        file: std::path::PathBuf::default(),
    };

    let builder = InvoiceBuilder::new(&args);

    let expected = InvoiceBuilder {
        project_hours_logged: HashMap::new(),
        pay_rate: 0.0,
        gst_rate: 10.0,
    };
    assert_eq!(builder, expected);
}

#[test]
fn test_build_no_hours() {
    let args = Args {
        pay_rate: 0.0,
        gst: None,
        file: std::path::PathBuf::default(),
    };
    let builder = InvoiceBuilder::new(&args);

    let invoice = builder.build();

    let empty_invoice = Invoice {
        project_hours_logged: HashMap::new(),
        total_time: 0.0,
        subtotal: 0.0,
        gst: 0.0,
        total: 0.0,

        gst_rate: 0.0,
        pay_rate: 0.0,
    };
    assert_eq!(invoice, empty_invoice)
}

#[test]
fn test_manual_hours() {
    let args = Args {
        pay_rate: 25.0,
        gst: Some(0.08),
        file: std::path::PathBuf::default(),
    };

    let invoice = InvoiceBuilder::new(&args)
        .add_project_duration("test_project_1", &Duration::hours(13))
        .add_project_duration("test_project_2", &Duration::hours(6))
        .build();

    let expected_map = HashMap::from([
        ("test_project_1".to_owned(), 13.0),
        ("test_project_2".to_owned(), 6.0),
    ]);
    let expected_invoice = Invoice {
        project_hours_logged: expected_map,
        total_time: 19.0,
        subtotal: 475.0,
        gst: 38.0,
        total: 513.0,

        gst_rate: 0.08,
        pay_rate: 25.0,
    };
    assert_eq!(invoice, expected_invoice)
}

#[test]
fn test_manual_hours_overlap() {
    let args = Args {
        pay_rate: 25.0,
        gst: Some(0.08),
        file: std::path::PathBuf::default(),
    };

    let invoice = InvoiceBuilder::new(&args)
        .add_project_duration("test_project_1", &Duration::hours(13))
        .add_project_duration("test_project_1", &Duration::hours(6))
        .build();

    let expected_map = HashMap::from([("test_project_1".to_owned(), 19.0)]);
    let expected_invoice = Invoice {
        project_hours_logged: expected_map,
        total_time: 19.0,
        subtotal: 475.0,
        gst: 38.0,
        total: 513.0,

        gst_rate: 0.08,
        pay_rate: 25.0,
    };
    assert_eq!(invoice, expected_invoice)
}

#[test]
fn test_collect_time_entries() {
    let args = Args {
        pay_rate: 25.0,
        gst: Some(0.08),
        file: std::path::PathBuf::default(),
    };

    let entries = vec![
        ("test_project_1".to_owned(), Duration::hours(13)),
        ("test_project_2".to_owned(), Duration::hours(6)),
    ];
    let invoice = InvoiceBuilder::new(&args)
        .collect_time_entries(entries.as_slice())
        .build();

    let expected_map = HashMap::from([
        ("test_project_1".to_owned(), 13.0),
        ("test_project_2".to_owned(), 6.0),
    ]);
    let expected_invoice = Invoice {
        project_hours_logged: expected_map,
        total_time: 19.0,
        subtotal: 475.0,
        gst: 38.0,
        total: 513.0,

        gst_rate: 0.08,
        pay_rate: 25.0,
    };
    assert_eq!(invoice, expected_invoice)
}

#[test]
fn test_parse_valid_time() -> anyhow::Result<()> {
    const TIME_STR: &str = "10:05:16";

    let duration = InvoiceBuilder::parse_duration_str(TIME_STR)?;

    let expected_duration = Duration::hours(10) + Duration::minutes(05) + Duration::seconds(16);
    assert_eq!(duration, expected_duration);

    Ok(())
}

#[test]
fn test_parse_extra_long_time() -> anyhow::Result<()> {
    const TIME_STR: &str = "10:05:16:18:342";

    let duration = InvoiceBuilder::parse_duration_str(TIME_STR)?;

    let expected_duration = Duration::hours(10) + Duration::minutes(05) + Duration::seconds(16);
    assert_eq!(duration, expected_duration);

    Ok(())
}

#[test]
fn test_parse_invalid_time() {
    const TIME_STR: &str = "10::16";

    let duration = InvoiceBuilder::parse_duration_str(TIME_STR);

    assert!(duration.is_err());
}
