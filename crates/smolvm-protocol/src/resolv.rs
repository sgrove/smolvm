//! Guest `/etc/resolv.conf` contents, rendered in one place.
//!
//! The agent writes resolv.conf from several paths (the TSI boot repair, the
//! persistent-overlay refresh on every exec, and the virtio-net setup). They
//! all render through [`render`] so the nameserver order and the `--dns-option`
//! resolver options can never drift between them; the host validates options
//! with the same [`validate_option`] the agent applies.

/// The public resolvers used when the host names none, in order.
pub const DEFAULT_NAMESERVERS: [&str; 2] = ["1.1.1.1", "8.8.8.8"];

/// At most this many options (glibc honors a handful; more is a mistake).
pub const MAX_OPTIONS: usize = 8;

/// Accept one glibc resolver option (`resolv.conf(5)` `options`), or say why
/// not. Boolean options take no value; `ndots`, `timeout` and `attempts` take
/// a small integer within glibc's own caps.
pub fn validate_option(option: &str) -> Result<(), String> {
    let (name, value) = match option.split_once(':') {
        Some((name, value)) => (name, Some(value)),
        None => (option, None),
    };
    let bound = match name {
        "ndots" => Some(15),
        "timeout" => Some(30),
        "attempts" => Some(5),
        "rotate" | "no-check-names" | "inet6" | "edns0" | "single-request"
        | "single-request-reopen" | "no-tld-query" | "use-vc" | "no-reload" | "trust-ad"
        | "no-aaaa" => None,
        _ => return Err(format!("unsupported resolver option `{option}`")),
    };
    match (bound, value) {
        (None, None) => Ok(()),
        (None, Some(_)) => Err(format!("resolver option `{name}` takes no value")),
        (Some(_), None) => Err(format!("resolver option `{name}` needs a value, e.g. `{name}:1`")),
        (Some(max), Some(value)) => match value.parse::<u32>() {
            Ok(n) if n <= max && (name == "ndots" || n >= 1) => Ok(()),
            _ => Err(format!("resolver option `{option}` is out of range")),
        },
    }
}

/// Parse the space-separated [`crate::guest_env::DNS_OPTIONS`] value; invalid
/// or excess tokens are dropped (each reported), never written.
pub fn parse_options(value: &str) -> (Vec<String>, Vec<String>) {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    for option in value.split_whitespace() {
        match validate_option(option) {
            Ok(()) if accepted.len() < MAX_OPTIONS => accepted.push(option.to_string()),
            Ok(()) => rejected.push(format!("`{option}` exceeds {MAX_OPTIONS} options")),
            Err(reason) => rejected.push(reason),
        }
    }
    (accepted, rejected)
}

/// The full file: one `nameserver` line per server in order, then one
/// `options` line when any options are given.
pub fn render<S: AsRef<str>>(nameservers: &[S], options: &[String]) -> String {
    let mut out = String::new();
    for server in nameservers {
        out.push_str("nameserver ");
        out.push_str(server.as_ref().trim());
        out.push('\n');
    }
    if !options.is_empty() {
        out.push_str("options ");
        out.push_str(&options.join(" "));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_servers_in_order_then_one_options_line() {
        let options = vec!["timeout:1".to_string(), "attempts:3".to_string(), "rotate".to_string(), "use-vc".to_string()];
        assert_eq!(
            render(&DEFAULT_NAMESERVERS, &options),
            "nameserver 1.1.1.1\nnameserver 8.8.8.8\noptions timeout:1 attempts:3 rotate use-vc\n"
        );
        assert_eq!(render(&["10.0.0.2"], &[]), "nameserver 10.0.0.2\n");
    }

    #[test]
    fn options_are_validated_against_glibc_names_and_caps() {
        for ok in ["timeout:1", "attempts:5", "ndots:0", "rotate", "use-vc", "edns0", "trust-ad"] {
            assert_eq!(validate_option(ok), Ok(()), "{ok}");
        }
        for bad in ["timeout:0", "attempts:6", "timeout", "rotate:1", "ndots:x", "evil\nnameserver 6.6.6.6", "", "options"] {
            assert!(validate_option(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn parse_drops_invalid_and_excess_tokens_and_reports_them() {
        let (accepted, rejected) = parse_options("timeout:1 bogus attempts:3");
        assert_eq!(accepted, vec!["timeout:1", "attempts:3"]);
        assert_eq!(rejected.len(), 1);
        let many = std::iter::repeat_n("rotate", MAX_OPTIONS + 1).collect::<Vec<_>>().join(" ");
        let (accepted, rejected) = parse_options(&many);
        assert_eq!(accepted.len(), MAX_OPTIONS);
        assert_eq!(rejected.len(), 1);
    }
}
