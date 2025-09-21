fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = inaturalist_oauth::get_api_token()?;
    println!("Got token: {}", token);
    Ok(())
}