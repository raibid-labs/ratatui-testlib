//! Simple CLI tool for auditing and scaffolding tests.
//!
//! Usage:
//!   cargo run --example audit_cli -- scan <directory>
//!   cargo run --example audit_cli -- report <directory>
//!   cargo run --example audit_cli -- scaffold <file> [--harness <type>]

use ratatui_testlib::audit::{HarnessType, ScaffoldConfig, TestAuditor};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];
    let path = Path::new(&args[2]);

    match command.as_str() {
        "scan" => cmd_scan(path)?,
        "report" => cmd_report(path)?,
        "scaffold" => {
            let harness = parse_harness_arg(&args);
            cmd_scaffold(path, harness)?;
        }
        "summary" => cmd_summary(path)?,
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
        }
    }

    Ok(())
}

fn cmd_scan(path: &Path) -> std::io::Result<()> {
    println!("Scanning for placeholder tests in: {}", path.display());
    println!();

    let placeholders = if path.is_file() {
        TestAuditor::find_placeholders_in_file(path)?
    } else {
        TestAuditor::find_placeholders_in_dir(path)?
    };

    if placeholders.is_empty() {
        println!("No placeholder tests found!");
        return Ok(());
    }

    println!("Found {} placeholder tests:\n", placeholders.len());

    for test in placeholders {
        println!("  {}:{}", test.file.display(), test.line);
        println!("    Function: {}", test.function_name);
        println!("    Pattern:  {}", test.pattern.as_str());
        println!("    Subject:  {}", test.inferred_subject);
        println!();
    }

    Ok(())
}

fn cmd_report(path: &Path) -> std::io::Result<()> {
    let placeholders = if path.is_file() {
        TestAuditor::find_placeholders_in_file(path)?
    } else {
        TestAuditor::find_placeholders_in_dir(path)?
    };

    let report = TestAuditor::generate_report(&placeholders);
    println!("{}", report);

    Ok(())
}

fn cmd_summary(path: &Path) -> std::io::Result<()> {
    let placeholders = if path.is_file() {
        TestAuditor::find_placeholders_in_file(path)?
    } else {
        TestAuditor::find_placeholders_in_dir(path)?
    };

    let summary = TestAuditor::summarize(&placeholders);

    println!("Audit Summary");
    println!("=============");
    println!();
    println!("Files scanned:       {}", summary.files_scanned);
    println!("Placeholders found:  {}", summary.placeholders_found);
    println!();

    if !summary.by_pattern.is_empty() {
        println!("Breakdown by pattern:");
        let mut patterns: Vec<_> = summary.by_pattern.iter().collect();
        patterns.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        for (pattern, count) in patterns {
            println!("  {:30} {:>3} tests", pattern, count);
        }
    }

    Ok(())
}

fn cmd_scaffold(path: &Path, harness: HarnessType) -> std::io::Result<()> {
    let placeholders = if path.is_file() {
        TestAuditor::find_placeholders_in_file(path)?
    } else {
        TestAuditor::find_placeholders_in_dir(path)?
    };

    if placeholders.is_empty() {
        println!("No placeholder tests found to scaffold.");
        return Ok(());
    }

    let config = ScaffoldConfig {
        harness,
        include_setup_teardown: true,
        generate_comments: true,
        include_error_handling: true,
    };

    let scaffolded = TestAuditor::scaffold_test_file(&placeholders, &config);
    println!("{}", scaffolded);

    Ok(())
}

fn parse_harness_arg(args: &[String]) -> HarnessType {
    for i in 0..args.len() {
        if args[i] == "--harness" && i + 1 < args.len() {
            return match args[i + 1].to_lowercase().as_str() {
                "async" => HarnessType::AsyncTuiTestHarness,
                "bevy" => HarnessType::BevyTuiTestHarness,
                "scarab" => HarnessType::ScarabTestHarness,
                _ => HarnessType::TuiTestHarness,
            };
        }
    }
    HarnessType::TuiTestHarness
}

fn print_usage() {
    println!("Test Audit CLI");
    println!();
    println!("Usage:");
    println!("  audit_cli scan <path>        - Scan for placeholder tests");
    println!("  audit_cli report <path>      - Generate markdown audit report");
    println!("  audit_cli summary <path>     - Show summary statistics");
    println!("  audit_cli scaffold <path>    - Generate scaffolded test file");
    println!();
    println!("Options:");
    println!("  --harness <type>             - Harness type: tui, async, bevy, scarab");
    println!();
    println!("Examples:");
    println!("  audit_cli scan tests/");
    println!("  audit_cli report tests/ > audit-report.md");
    println!("  audit_cli scaffold tests/my_test.rs --harness async");
}
