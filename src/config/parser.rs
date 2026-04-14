use crate::error::AppError;

#[derive(Debug, Clone, PartialEq)]
pub struct OvpnConfig {
    pub raw_content: String,
    pub remote_servers: Vec<RemoteServer>,
    pub needs_auth_user_pass: bool,
    pub static_challenge: Option<StaticChallenge>,
    pub has_inline_ca: bool,
    pub has_inline_cert: bool,
    pub has_inline_key: bool,
    pub ca_path: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub tls_mode: Option<TlsMode>,
    pub protocol: Option<String>,
    pub is_client: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoteServer {
    pub host: String,
    pub port: u16,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticChallenge {
    pub text: String,
    pub echo: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TlsMode {
    TlsAuth {
        key_path: String,
        direction: Option<String>,
    },
    TlsCrypt {
        key_path: String,
    },
}

/// Parse an OpenVPN config file content into an `OvpnConfig` struct.
///
/// Extracts only the directives the GUI needs to know about (auth requirements,
/// server info, certificate presence). Does NOT validate the config — OpenVPN
/// itself handles that.
pub fn parse_ovpn(content: &str) -> Result<OvpnConfig, AppError> {
    let mut config = OvpnConfig {
        raw_content: content.to_string(),
        remote_servers: Vec::new(),
        needs_auth_user_pass: false,
        static_challenge: None,
        has_inline_ca: false,
        has_inline_cert: false,
        has_inline_key: false,
        ca_path: None,
        cert_path: None,
        key_path: None,
        tls_mode: None,
        protocol: None,
        is_client: false,
    };

    let mut inside_block: Option<&str> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        // Handle inline block boundaries
        if let Some(block_name) = trimmed.strip_prefix("</")
            && let Some(block_name) = block_name.strip_suffix('>')
        {
            match block_name {
                "ca" => config.has_inline_ca = true,
                "cert" => config.has_inline_cert = true,
                "key" => config.has_inline_key = true,
                _ => {}
            }
            inside_block = None;
            continue;
        }

        if trimmed.starts_with('<') && trimmed.ends_with('>') && !trimmed.starts_with("</") {
            inside_block = Some(&trimmed[1..trimmed.len() - 1]);
            continue;
        }

        // Skip content inside inline blocks
        if inside_block.is_some() {
            continue;
        }

        // Parse directives
        let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
        let directive = parts[0];
        let args = parts.get(1).map(|s| s.trim());

        match directive {
            "client" => {
                config.is_client = true;
            }
            "auth-user-pass" => {
                // If there's a file argument, credentials come from file — no GUI prompt
                // If no argument, GUI must prompt for username/password
                config.needs_auth_user_pass = args.is_none_or(|a| a.is_empty());
            }
            "static-challenge" => {
                if let Some(args_str) = args {
                    config.static_challenge = parse_static_challenge(args_str);
                }
            }
            "remote" => {
                if let Some(args_str) = args
                    && let Some(server) = parse_remote(args_str)
                {
                    config.remote_servers.push(server);
                }
            }
            "proto" => {
                if let Some(proto) = args {
                    config.protocol = Some(proto.to_string());
                }
            }
            "ca" => {
                if let Some(path) = args {
                    config.ca_path = Some(path.to_string());
                }
            }
            "cert" => {
                if let Some(path) = args {
                    config.cert_path = Some(path.to_string());
                }
            }
            "key" => {
                if let Some(path) = args {
                    config.key_path = Some(path.to_string());
                }
            }
            "tls-auth" => {
                if let Some(args_str) = args {
                    let tls_parts: Vec<&str> = args_str.split_whitespace().collect();
                    if let Some(&key_path) = tls_parts.first() {
                        config.tls_mode = Some(TlsMode::TlsAuth {
                            key_path: key_path.to_string(),
                            direction: tls_parts.get(1).map(|s| s.to_string()),
                        });
                    }
                }
            }
            "tls-crypt" => {
                if let Some(args_str) = args {
                    let key_path = args_str.split_whitespace().next().unwrap_or(args_str);
                    config.tls_mode = Some(TlsMode::TlsCrypt {
                        key_path: key_path.to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(config)
}

/// Parse `static-challenge "prompt text" echo_flag`
fn parse_static_challenge(args: &str) -> Option<StaticChallenge> {
    // The text is enclosed in double quotes, possibly followed by 0 or 1
    let args = args.trim();

    if !args.starts_with('"') {
        return None;
    }

    // Find the closing quote, handling escaped quotes
    let chars: Vec<char> = args.chars().collect();
    let mut text = String::new();
    let mut i = 1; // skip opening quote
    let mut prev_was_escape = false;

    while i < chars.len() {
        let ch = chars[i];
        if prev_was_escape {
            text.push(ch);
            prev_was_escape = false;
        } else if ch == '\\' {
            prev_was_escape = true;
        } else if ch == '"' {
            break;
        } else {
            text.push(ch);
        }
        i += 1;
    }

    // Parse echo flag from remainder after the closing quote
    let remainder = &args[i + 1..].trim();
    let echo = remainder.starts_with('1');

    Some(StaticChallenge { text, echo })
}

/// Parse `host port [protocol]` from a remote directive
fn parse_remote(args: &str) -> Option<RemoteServer> {
    let parts: Vec<&str> = args.split_whitespace().collect();

    let host = parts.first()?.to_string();
    let port = parts.get(1).and_then(|p| p.parse::<u16>().ok()).unwrap_or(1194);
    let protocol = parts.get(2).map(|s| s.to_string());

    Some(RemoteServer {
        host,
        port,
        protocol,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_client() {
        let content = r#"
client
proto udp
remote vpn.example.com 1194
ca /etc/openvpn/ca.crt
cert /etc/openvpn/client.crt
key /etc/openvpn/client.key
"#;
        let config = parse_ovpn(content).unwrap();
        assert!(config.is_client);
        assert_eq!(config.protocol.as_deref(), Some("udp"));
        assert_eq!(config.remote_servers.len(), 1);
        assert_eq!(config.remote_servers[0].host, "vpn.example.com");
        assert_eq!(config.remote_servers[0].port, 1194);
        assert_eq!(config.ca_path.as_deref(), Some("/etc/openvpn/ca.crt"));
        assert_eq!(config.cert_path.as_deref(), Some("/etc/openvpn/client.crt"));
        assert_eq!(config.key_path.as_deref(), Some("/etc/openvpn/client.key"));
        assert!(!config.needs_auth_user_pass);
        assert!(config.static_challenge.is_none());
    }

    #[test]
    fn test_parse_auth_user_pass() {
        let content = r#"
client
auth-user-pass
remote vpn.example.com 443 tcp
"#;
        let config = parse_ovpn(content).unwrap();
        assert!(config.needs_auth_user_pass);
        assert!(config.static_challenge.is_none());
    }

    #[test]
    fn test_parse_auth_user_pass_with_file() {
        let content = r#"
client
auth-user-pass /path/to/credentials.txt
remote vpn.example.com 1194
"#;
        let config = parse_ovpn(content).unwrap();
        assert!(!config.needs_auth_user_pass);
    }

    #[test]
    fn test_parse_static_challenge() {
        let content = r#"
client
auth-user-pass
static-challenge "Enter OTP token" 1
remote vpn.example.com 1194
"#;
        let config = parse_ovpn(content).unwrap();
        assert!(config.needs_auth_user_pass);
        let challenge = config.static_challenge.unwrap();
        assert_eq!(challenge.text, "Enter OTP token");
        assert!(challenge.echo);
    }

    #[test]
    fn test_parse_static_challenge_no_echo() {
        let content = r#"
client
auth-user-pass
static-challenge "Enter your PIN" 0
remote vpn.example.com 1194
"#;
        let config = parse_ovpn(content).unwrap();
        let challenge = config.static_challenge.unwrap();
        assert_eq!(challenge.text, "Enter your PIN");
        assert!(!challenge.echo);
    }

    #[test]
    fn test_parse_inline_certs() {
        let content = r#"
client
remote vpn.example.com 1194

<ca>
-----BEGIN CERTIFICATE-----
MIIBozCCAUmgAwIBAgIJAP...
-----END CERTIFICATE-----
</ca>

<cert>
-----BEGIN CERTIFICATE-----
MIIBozCCAUmgAwIBAgIJAP...
-----END CERTIFICATE-----
</cert>

<key>
-----BEGIN PRIVATE KEY-----
MIIBozCCAUmgAwIBAgIJAP...
-----END PRIVATE KEY-----
</key>
"#;
        let config = parse_ovpn(content).unwrap();
        assert!(config.has_inline_ca);
        assert!(config.has_inline_cert);
        assert!(config.has_inline_key);
        assert!(config.ca_path.is_none());
        assert!(config.cert_path.is_none());
        assert!(config.key_path.is_none());
    }

    #[test]
    fn test_parse_multiple_remotes() {
        let content = r#"
client
remote server1.example.com 1194 udp
remote server2.example.com 443 tcp
remote server3.example.com 8080
"#;
        let config = parse_ovpn(content).unwrap();
        assert_eq!(config.remote_servers.len(), 3);
        assert_eq!(config.remote_servers[0].host, "server1.example.com");
        assert_eq!(config.remote_servers[0].port, 1194);
        assert_eq!(config.remote_servers[0].protocol.as_deref(), Some("udp"));
        assert_eq!(config.remote_servers[1].host, "server2.example.com");
        assert_eq!(config.remote_servers[1].port, 443);
        assert_eq!(config.remote_servers[1].protocol.as_deref(), Some("tcp"));
        assert_eq!(config.remote_servers[2].host, "server3.example.com");
        assert_eq!(config.remote_servers[2].port, 8080);
        assert!(config.remote_servers[2].protocol.is_none());
    }

    #[test]
    fn test_parse_comments_and_semicolons() {
        let content = r#"
# This is a comment
; This is also a comment
client
# auth-user-pass should be ignored here
remote vpn.example.com 1194
"#;
        let config = parse_ovpn(content).unwrap();
        assert!(config.is_client);
        assert!(!config.needs_auth_user_pass);
        assert_eq!(config.remote_servers.len(), 1);
    }

    #[test]
    fn test_parse_tls_auth() {
        let content = r#"
client
remote vpn.example.com 1194
tls-auth ta.key 1
"#;
        let config = parse_ovpn(content).unwrap();
        assert_eq!(
            config.tls_mode,
            Some(TlsMode::TlsAuth {
                key_path: "ta.key".to_string(),
                direction: Some("1".to_string()),
            })
        );
    }

    #[test]
    fn test_parse_tls_crypt() {
        let content = r#"
client
remote vpn.example.com 1194
tls-crypt tc.key
"#;
        let config = parse_ovpn(content).unwrap();
        assert_eq!(
            config.tls_mode,
            Some(TlsMode::TlsCrypt {
                key_path: "tc.key".to_string(),
            })
        );
    }

    #[test]
    fn test_parse_remote_default_port() {
        let content = r#"
client
remote vpn.example.com
"#;
        let config = parse_ovpn(content).unwrap();
        assert_eq!(config.remote_servers[0].port, 1194);
    }

    #[test]
    fn test_parse_empty_content() {
        let config = parse_ovpn("").unwrap();
        assert!(!config.is_client);
        assert!(config.remote_servers.is_empty());
        assert!(!config.needs_auth_user_pass);
    }

    #[test]
    fn test_parse_static_challenge_escaped_quotes() {
        let content = r#"
client
auth-user-pass
static-challenge "Enter your \"secure\" token" 1
"#;
        let config = parse_ovpn(content).unwrap();
        let challenge = config.static_challenge.unwrap();
        assert_eq!(challenge.text, r#"Enter your "secure" token"#);
        assert!(challenge.echo);
    }
}
