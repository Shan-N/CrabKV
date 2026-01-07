use super::ParsedCommand;

pub fn parse_command(input: &str) -> Option<ParsedCommand> {
    match input.split_whitespace().collect::<Vec<_>>().as_slice() {
        ["SET", key, value] => Some(ParsedCommand::Set {
            key: key.to_string(),
            value: value.to_string(),
        }),
        ["SETEX", key, value, ttl] => Some(ParsedCommand::SetEx {
            key: key.to_string(),
            value: value.to_string(),
            ttl: ttl.parse::<u64>().unwrap(),
        }),
        ["GET", key] => Some(ParsedCommand::Get {
            key: key.to_string(),
        }),
        ["DEL", key] => Some(ParsedCommand::Del {
            key: key.to_string(),
        }),
        ["EX", key] => Some(ParsedCommand::Ex {
            key: key.to_string(),
        }),
        ["EXPIRE", key, ttl] => Some(ParsedCommand::Expire {
            key: key.to_string(),
            ttl: ttl.parse::<u64>().unwrap(),
        }),
        ["TTL", key] => Some(ParsedCommand::Ttl {
            key: key.to_string(),
        }),
        _ => None,
    }
}
