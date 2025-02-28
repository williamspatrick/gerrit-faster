use dotenv::dotenv;

fn main() {
    dotenv().ok();

    println!("Hello, {}!", std::env::var("GERRIT_USERNAME").expect("USERNAME must be set"));
}
