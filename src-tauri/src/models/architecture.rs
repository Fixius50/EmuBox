use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    X86_64,
    Aarch64,
    Unsupported,
}

impl Architecture {
    pub fn parse(value: &str) -> Self {
        match value {
            "x86_64" => Self::X86_64,
            "aarch64" | "arm64" => Self::Aarch64,
            _ => Self::Unsupported,
        }
    }

    pub fn current() -> Self {
        Self::parse(std::env::consts::ARCH)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Unsupported => "unsupported",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architecture_matrix() {
        for (input, expected) in [
            ("x86_64", Architecture::X86_64),
            ("aarch64", Architecture::Aarch64),
            ("arm64", Architecture::Aarch64),
            ("armv7l", Architecture::Unsupported),
            ("riscv64", Architecture::Unsupported),
        ] {
            assert_eq!(Architecture::parse(input), expected);
            assert_eq!(serde_json::to_value(expected).unwrap(), expected.as_str());
        }
    }
}