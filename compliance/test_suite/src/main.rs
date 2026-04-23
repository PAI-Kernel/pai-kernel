use compliance_test_suite::run;

fn main() {
    let results = run();
    let any_fail = results.iter().any(|r| !r.pass);

    let json = serde_json::to_string_pretty(&results).expect("serialize report");
    println!("{}", json);

    if any_fail {
        std::process::exit(1);
    }
}
