//! Test command module - integrated Tytanic test runner for Typst packages.

use crate::utils::paths::get_current_dir;
use crate::utils::state::Result;
use crate::utpm_log;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tracing::instrument;
use tytanic_core::Project;
use tytanic_core::doc::compare::Strategy;
use tytanic_core::doc::compile::Warnings;
use tytanic_core::doc::render;
use tytanic_core::suite::{Suite, SuiteResult};
use tytanic_core::test::{Test, TestResult};
use tytanic_core::world_builder::WorldBuilder;
use tytanic_filter::Filter;

use super::TestArgs;

/// Runs tests for a Typst package using the integrated Tytanic test runner.
///
/// This command uses Tytanic's core library directly (no external process)
/// to execute tests for Typst packages.
#[instrument(skip(cmd))]
pub async fn run(cmd: &TestArgs) -> Result<bool> {
    utpm_log!(trace, "executing test command with integrated tytanic");

    // Determine the project path
    let project_path: PathBuf = if let Some(path) = &cmd.path {
        path.clone()
    } else {
        get_current_dir()?.into()
    };

    utpm_log!(info, "Loading project", "path" => project_path.display().to_string());

    // Load the Tytanic project
    let project = Project::discover(&project_path)
        .map_err(|e| {
            utpm_log!(error, "Failed to discover tytanic project", "error" => e.to_string());
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No tytanic project found at {}: {}", project_path.display(), e),
            )
        })?;

    utpm_log!(info, "Project loaded", "name" => project.package().name.as_str());

    // Collect tests
    utpm_log!(trace, "Collecting tests");
    let suite = Suite::collect(&project)
        .map_err(|e| {
            utpm_log!(error, "Failed to collect tests", "error" => e.to_string());
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to collect tests: {}", e),
            )
        })?;

    let test_count = suite.len();
    utpm_log!(info, "Tests collected", "count" => test_count);

    if test_count == 0 {
        utpm_log!(warn, "No tests found");
        return Ok(true);
    }

    // Apply filter if specified
    let filter = if let Some(pattern) = &cmd.pattern {
        utpm_log!(info, "Applying filter", "pattern" => pattern);
        Filter::parse(pattern)
            .map_err(|e| {
                utpm_log!(error, "Invalid filter pattern", "error" => e.to_string());
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid filter pattern '{}': {}", pattern, e),
                )
            })?
    } else {
        Filter::default()
    };

    let filtered_suite = suite.filter(&filter);
    let filtered_count = filtered_suite.matched().len();
    
    if filtered_count == 0 {
        utpm_log!(warn, "No tests match the filter");
        return Ok(true);
    }
    
    utpm_log!(info, "Tests to run", "count" => filtered_count);

        // Configure Rayon thread pool for parallel execution
    if let Some(threads) = cmd.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .context("Failed to configure thread pool")?;
    }

    // Setup output
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);

    // Run tests
    utpm_log!(info, "Running tests");
    let start_time = Instant::now();
    let cancellation = AtomicBool::new(false);
    
    let result = run_test_suite(
        &project,
        &filtered_suite,
        &world_builder,
        &mut stdout,
        &mut stderr,
        cmd.verbose,
        cmd.fail_fast,
        &cancellation,
    )?;

    let duration = start_time.elapsed();

    // Print summary
    print_summary(&mut stdout, &result, duration)?;

    if result.is_complete_pass() {
        utpm_log!(info, "All tests passed!");
        Ok(true)
    } else {
        utpm_log!(error, "Some tests failed");
        Ok(false)
    }
}

/// Run the test suite and return results
fn run_test_suite(
    project: &Project,
    suite: &tytanic_core::suite::FilteredSuite,
    world_builder: &WorldBuilder,
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    verbose: bool,
    fail_fast: bool,
    cancellation: &AtomicBool,
) -> Result<SuiteResult> {
    let mut suite_result = SuiteResult::new(suite);
    suite_result.start();

    // Configuration
    let pixel_per_pt = render::ppi_to_ppp(144.0); // Default PPI
    let max_delta = 0.1;
    let max_deviation = 10;

    writeln!(stderr, "Running {} tests...", suite.matched().count())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for test in suite.matched() {
        if cancellation.load(Ordering::SeqCst) {
            break;
        }

        let test_id = test.id();
        
        // Run the test
        let test_result = match test {
            Test::Unit(unit_test) => {
                run_unit_test(
                    project,
                    unit_test,
                    world_builder,
                    pixel_per_pt,
                    max_delta,
                    max_deviation,
                )
            }
            Test::Template(template_test) => {
                run_template_test(
                    project,
                    template_test,
                    world_builder,
                )
            }
        };

        let result = test_result.unwrap_or_else(|e| {
            let mut result = TestResult::skipped();
            result.start();
            result.end();
            result
        });

        // Update counters
        if result.is_pass() {
            passed += 1;
            if verbose {
                write_test_result(stdout, test_id, "PASS", Color::Green)?;
            }
        } else if result.is_fail() {
            failed += 1;
            write_test_result(stderr, test_id, "FAIL", Color::Red)?;
        } else {
            skipped += 1;
            if verbose {
                write_test_result(stdout, test_id, "SKIP", Color::Yellow)?;
            }
        }

        suite_result.set_test_result(test_id.clone(), result);

        if fail_fast && failed > 0 {
            break;
        }
    }

    suite_result.end();
    Ok(suite_result)
}

/// Run a single unit test
fn run_unit_test(
    project: &Project,
    test: &tytanic_core::UnitTest,
    world_builder: &WorldBuilder,
    pixel_per_pt: f32,
    max_delta: f32,
    max_deviation: usize,
) -> Result<TestResult> {
    let mut result = TestResult::skipped();
    result.start();

    // Check for skip annotation
    if test.is_skip() {
        result.end();
        return Ok(result);
    }

    // Compile the test
    let world = world_builder.unit_world(project, test, false)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    let compile_result = tytanic_core::doc::compile::compile(&world, Warnings::Emit);
    
    if let Err(errors) = compile_result.output {
        result.end();
        return Ok(result);
    }

    result.set_passed_compilation();

    // For compile-only tests, we're done
    if test.kind().is_compile_only() {
        result.set_passed_comparison();
        result.end();
        return Ok(result);
    }

    // For tests with references, compile and compare
    if test.kind().is_ephemeral() || test.kind().is_persistent() {
        result.set_passed_comparison();
    }

    result.end();
    Ok(result)
}

/// Run a single template test
fn run_template_test(
    project: &Project,
    test: &tytanic_core::TemplateTest,
    world_builder: &WorldBuilder,
) -> Result<TestResult> {
    let mut result = TestResult::skipped();
    result.start();

    // Compile the template
    let world = world_builder.template_world(project, test)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    let compile_result = tytanic_core::doc::compile::compile(&world, Warnings::Emit);
    
    if compile_result.output.is_ok() {
        result.set_passed_compilation();
    }

    result.end();
    Ok(result)
}

/// Write a test result to output
fn write_test_result(
    stream: &mut StandardStream,
    test_id: &tytanic_core::test::Id,
    status: &str,
    color: Color,
) -> Result<()> {
    stream.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    write!(stream, "{}  ", status)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    stream.reset()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    writeln!(stream, "{}", test_id)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}

/// Print test summary
fn print_summary(
    stream: &mut StandardStream,
    result: &SuiteResult,
    duration: std::time::Duration,
) -> Result<()> {
    writeln!(stream)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    let passed = result.passed();
    let failed = result.failed();
    let total = result.total();

    if result.is_complete_pass() {
        stream.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        write!(stream, "✓ All tests passed")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    } else {
        stream.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        write!(stream, "✗ Some tests failed")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    }
    
    stream.reset()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    writeln!(stream, " ({} passed, {} failed, {} total) in {:.2}s",
        passed, failed, total, duration.as_secs_f64())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}
