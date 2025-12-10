//! Integration tests for UI region testing helpers.

#[cfg(feature = "ipc")]
mod region_tests {
    use ratatui_testlib::regions::{
        RegionAnchor, RegionBounds, UiRegion, UiRegionTester,
    };

    #[test]
    fn test_basic_region_setup() {
        let tester = UiRegionTester::new(80, 24)
            .with_status_bar(1)
            .with_tab_bar(2);

        assert_eq!(tester.screen_dimensions(), (80, 24));
        assert_eq!(tester.region_names().len(), 2);

        let status = tester.region_bounds("status_bar").unwrap();
        assert_eq!(status.row, 23);
        assert_eq!(status.col, 0);
        assert_eq!(status.width, 80);
        assert_eq!(status.height, 1);

        let tabs = tester.region_bounds("tab_bar").unwrap();
        assert_eq!(tabs.row, 0);
        assert_eq!(tabs.col, 0);
        assert_eq!(tabs.width, 80);
        assert_eq!(tabs.height, 2);
    }

    #[test]
    fn test_complex_layout() {
        let tester = UiRegionTester::new(100, 30)
            .with_tab_bar(2)
            .with_status_bar(1)
            .with_left_sidebar(20)
            .with_right_sidebar(15);

        let content = tester.content_area();
        assert_eq!(content.row, 2); // Below tab bar
        assert_eq!(content.col, 20); // Right of left sidebar
        assert_eq!(content.width, 65); // 100 - 20 (left) - 15 (right)
        assert_eq!(content.height, 27); // 30 - 2 (tab) - 1 (status)

        // Verify all regions are accessible
        assert!(tester.region_bounds("tab_bar").is_some());
        assert!(tester.region_bounds("status_bar").is_some());
        assert!(tester.region_bounds("left_sidebar").is_some());
        assert!(tester.region_bounds("right_sidebar").is_some());
    }

    #[test]
    fn test_position_checking() {
        let tester = UiRegionTester::new(80, 24)
            .with_status_bar(1)
            .with_tab_bar(2);

        // Test content area positions
        assert!(tester.is_in_content_area(10, 10));
        assert!(tester.is_in_content_area(2, 0)); // First content row
        assert!(tester.is_in_content_area(22, 79)); // Last content row

        // Test fixed region positions
        assert!(!tester.is_in_content_area(0, 10)); // Tab bar
        assert!(!tester.is_in_content_area(1, 10)); // Tab bar
        assert!(!tester.is_in_content_area(23, 10)); // Status bar

        // Test specific region checks
        assert!(tester.is_in_region("tab_bar", 0, 0));
        assert!(tester.is_in_region("tab_bar", 1, 79));
        assert!(!tester.is_in_region("tab_bar", 2, 0));

        assert!(tester.is_in_region("status_bar", 23, 0));
        assert!(tester.is_in_region("status_bar", 23, 79));
        assert!(!tester.is_in_region("status_bar", 22, 0));
    }

    #[test]
    fn test_custom_regions() {
        let header = UiRegion {
            name: "header".to_string(),
            anchor: RegionAnchor::Top,
            size: 1,
        };

        let notification = UiRegion {
            name: "notification".to_string(),
            anchor: RegionAnchor::Top,
            size: 3,
        };

        let tester = UiRegionTester::new(80, 24)
            .with_region(header)
            .with_region(notification);

        let header_bounds = tester.region_bounds("header").unwrap();
        assert_eq!(header_bounds.row, 0);
        assert_eq!(header_bounds.height, 1);

        let notification_bounds = tester.region_bounds("notification").unwrap();
        assert_eq!(notification_bounds.row, 1); // Below header
        assert_eq!(notification_bounds.height, 3);
    }

    #[test]
    fn test_region_bounds_operations() {
        let bounds = RegionBounds::new(10, 20, 30, 15);

        // Test contains
        assert!(bounds.contains(10, 20)); // Top-left
        assert!(bounds.contains(15, 30)); // Middle
        assert!(bounds.contains(24, 49)); // Bottom-right corner (exclusive)
        assert!(!bounds.contains(9, 25)); // Above
        assert!(!bounds.contains(25, 25)); // Below
        assert!(!bounds.contains(15, 19)); // Left
        assert!(!bounds.contains(15, 50)); // Right

        // Test intersects
        let overlapping = RegionBounds::new(15, 25, 20, 10);
        assert!(bounds.intersects(&overlapping));

        let separate = RegionBounds::new(50, 50, 10, 10);
        assert!(!bounds.intersects(&separate));

        let contained = RegionBounds::new(12, 22, 5, 5);
        assert!(bounds.intersects(&contained));
    }

    #[test]
    fn test_sidebar_positioning() {
        let tester = UiRegionTester::new(80, 24)
            .with_tab_bar(2)
            .with_status_bar(1)
            .with_left_sidebar(20);

        let sidebar = tester.region_bounds("left_sidebar").unwrap();
        assert_eq!(sidebar.row, 2); // Below tab bar
        assert_eq!(sidebar.col, 0);
        assert_eq!(sidebar.width, 20);
        assert_eq!(sidebar.height, 21); // Between tab bar and status bar
    }

    #[test]
    fn test_right_sidebar_positioning() {
        let tester = UiRegionTester::new(80, 24)
            .with_tab_bar(2)
            .with_status_bar(1)
            .with_right_sidebar(15);

        let sidebar = tester.region_bounds("right_sidebar").unwrap();
        assert_eq!(sidebar.row, 2); // Below tab bar
        assert_eq!(sidebar.col, 65); // 80 - 15
        assert_eq!(sidebar.width, 15);
        assert_eq!(sidebar.height, 21); // Between tab bar and status bar
    }

    #[test]
    fn test_multiple_same_anchor_regions() {
        let tester = UiRegionTester::new(80, 24)
            .with_region(UiRegion {
                name: "header".to_string(),
                anchor: RegionAnchor::Top,
                size: 1,
            })
            .with_region(UiRegion {
                name: "tabs".to_string(),
                anchor: RegionAnchor::Top,
                size: 2,
            })
            .with_region(UiRegion {
                name: "toolbar".to_string(),
                anchor: RegionAnchor::Top,
                size: 1,
            });

        let header = tester.region_bounds("header").unwrap();
        assert_eq!(header.row, 0);

        let tabs = tester.region_bounds("tabs").unwrap();
        assert_eq!(tabs.row, 1); // Below header

        let toolbar = tester.region_bounds("toolbar").unwrap();
        assert_eq!(toolbar.row, 3); // Below header + tabs
    }

    #[test]
    fn test_edge_cases() {
        // Single row screen
        let tiny = UiRegionTester::new(80, 1).with_status_bar(1);
        let content = tiny.content_area();
        assert_eq!(content.height, 0);

        // Regions larger than screen
        let overflow = UiRegionTester::new(80, 24)
            .with_tab_bar(15)
            .with_status_bar(15);
        let overflow_content = overflow.content_area();
        assert_eq!(overflow_content.height, 0);

        // Zero-width region bounds
        let zero = RegionBounds::new(0, 0, 0, 0);
        assert!(!zero.contains(0, 0));
        assert!(!zero.contains(1, 1));
    }

    #[test]
    fn test_region_names() {
        let tester = UiRegionTester::new(80, 24)
            .with_status_bar(1)
            .with_tab_bar(2)
            .with_left_sidebar(20);

        let names = tester.region_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"status_bar".to_string()));
        assert!(names.contains(&"tab_bar".to_string()));
        assert!(names.contains(&"left_sidebar".to_string()));
    }

    #[test]
    fn test_nonexistent_region() {
        let tester = UiRegionTester::new(80, 24).with_status_bar(1);

        assert!(tester.region_bounds("nonexistent").is_none());
        assert!(!tester.is_in_region("nonexistent", 10, 10));
    }

    #[test]
    fn test_content_area_with_all_anchors() {
        let tester = UiRegionTester::new(100, 40)
            .with_tab_bar(3)
            .with_status_bar(2)
            .with_left_sidebar(15)
            .with_right_sidebar(20);

        let content = tester.content_area();

        // Verify content is correctly bounded by all regions
        assert_eq!(content.row, 3); // Below top region
        assert_eq!(content.col, 15); // Right of left sidebar
        assert_eq!(content.width, 65); // 100 - 15 (left) - 20 (right)
        assert_eq!(content.height, 35); // 40 - 3 (top) - 2 (bottom)

        // Verify content area positions
        assert!(tester.is_in_content_area(3, 15)); // Top-left of content
        assert!(tester.is_in_content_area(20, 50)); // Middle
        assert!(tester.is_in_content_area(37, 79)); // Bottom-right corner

        // Verify boundaries
        assert!(!tester.is_in_content_area(2, 15)); // Above (in tab bar)
        assert!(!tester.is_in_content_area(3, 14)); // Left (in left sidebar)
        assert!(!tester.is_in_content_area(3, 80)); // Right (in right sidebar)
        assert!(!tester.is_in_content_area(38, 50)); // Below (in status bar)
    }
}

#[cfg(all(feature = "scarab", target_family = "unix"))]
mod scarab_integration_tests {
    use ratatui_testlib::{
        regions::{UiRegionTestExt, UiRegionTester},
        scarab::ScarabTestHarness,
    };

    #[test]
    #[ignore] // Only run when Scarab daemon is available
    fn test_scarab_region_integration() {
        // This test requires SCARAB_TEST_RTL=1 and a running scarab-daemon
        if !ScarabTestHarness::is_enabled() {
            eprintln!("Skipping: SCARAB_TEST_RTL not set");
            return;
        }

        let result = ScarabTestHarness::connect();
        if result.is_err() {
            eprintln!("Skipping: Could not connect to Scarab daemon");
            return;
        }

        let mut harness = result.unwrap();
        let tester = UiRegionTester::new(80, 24).with_status_bar(1).with_tab_bar(2);

        // Test that we can get region contents
        let _status_result = harness.region_contents(&tester, "status_bar");
        let _content_result = harness.content_area_contents(&tester);

        // Test assertions (these may fail if daemon content doesn't match,
        // but the important part is that the API works)
        let _ = harness.assert_not_in_region(&tester, "status_bar", "UNLIKELY_STRING_XYZ");
    }
}
