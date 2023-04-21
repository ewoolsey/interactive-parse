# interactive-parse

**An interactive parser for JsonSchema types.

---

## Demo

---

https://user-images.githubusercontent.com/8366997/198078221-5fa01e97-a921-4441-b054-f75f4d1ff272.mp4

---

## Usage

---

```rust
    // Make sure you add these derives to your type
    #[derive(JsonSchema, Deserialize, Debug)]
    struct Git {
        subcommand: SubCommand,
        arg: String
    }

    #[derive(JsonSchema, Deserialize, Debug)]
    enum SubCommand {
        Commit {
            message: String
        },
        Clone {
            address: String
        }
    }

    // Bring the relevant traits into scope
    use interactive_parse::traits::InteractiveParseObj;


    fn main() {
        // Parse the type to an object
        let git = Git::parse_to_obj().unwrap();
        println!("{:?}", git);   
    }
```
---

## Looking for others to contribute

---

This is a simple approach at getting JsonSchema types to parse interactively using inquire. If you make improvements to this please submit a PR, and if you have any issues or bugs please submit an issue. I'm currently actively maintaining this project as a personal development tool.

In particular, this crate needs proper error handling. In most cases the crate will panic if it encounters an issue parsing. This is not ideal and could definitely be improved.

---
