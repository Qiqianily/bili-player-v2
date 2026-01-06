fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn first_test() {
        assert_eq!(2 + 2, 4);
    }
}
