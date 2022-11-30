# interactive-parse

**A work in progress parser for rust types that implement JsonSchema.

---

## Demo

---

https://user-images.githubusercontent.com/8366997/198078221-5fa01e97-a921-4441-b054-f75f4d1ff272.mp4

---

## Usage

---

```rust
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

    fn main() {
        let git = Git::interactive_parse().unwrap();
        println!("{:?}", git);   
    }
```
---

## Looking for others to contribute

---

This is a basic approach at getting JsonSchema types to parse interactively using inquire. If you make improvements to this please submit a PR, and if you have any issues or bugs please submit an issue. I'm currently actively maintaining this project as a personal development tool.

---
