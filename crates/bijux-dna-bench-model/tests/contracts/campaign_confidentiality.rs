use bijux_dna_bench_model::{validate_hpc_campaign_config_source, CampaignSlurm};

fn refuse_slurm(raw: &str) -> anyhow::Result<()> {
    let parsed: toml::Value = toml::from_str(raw)?;
    let value = parsed.get("slurm").ok_or_else(|| anyhow::anyhow!("slurm fixture absent"))?;
    let typed: CampaignSlurm = value.clone().try_into()?;
    assert_eq!(typed.account.as_deref(), Some("private-account"));
    let error = validate_hpc_campaign_config_source(raw)
        .err()
        .ok_or_else(|| anyhow::anyhow!("confidential slurm value was admitted"))?;
    assert!(error.to_string().contains("confidential slurm `account`"), "{error}");
    assert!(!error.to_string().contains("private-account"));
    Ok(())
}

#[test]
fn campaign_confidentiality_rejects_quoted_account_key() -> anyhow::Result<()> {
    refuse_slurm("[slurm]\n\"account\" = \"private-account\"\n")
}

#[test]
fn campaign_confidentiality_rejects_quoted_slurm_table() -> anyhow::Result<()> {
    refuse_slurm("[\"slurm\"]\naccount = \"private-account\"\n")
}

#[test]
fn campaign_confidentiality_rejects_escaped_account_key() -> anyhow::Result<()> {
    refuse_slurm("[slurm]\n\"acc\\u006funt\" = \"private-account\"\n")
}

#[test]
fn campaign_confidentiality_rejects_inline_slurm_table() -> anyhow::Result<()> {
    refuse_slurm("slurm = { account = \"private-account\" }\n")
}

#[test]
fn campaign_confidentiality_rejects_escaped_secret_key() -> anyhow::Result<()> {
    let raw = "\"to\\u006ben\" = \"private-token\"\n";
    let parsed: toml::Value = toml::from_str(raw)?;
    assert_eq!(parsed.get("token").and_then(toml::Value::as_str), Some("private-token"));
    let error = validate_hpc_campaign_config_source(raw)
        .err()
        .ok_or_else(|| anyhow::anyhow!("confidential token was admitted"))?;
    assert!(error.to_string().contains("sensitive key `token`"), "{error}");
    assert!(!error.to_string().contains("private-token"));
    Ok(())
}

#[test]
fn campaign_confidentiality_rejects_inline_nested_secret() -> anyhow::Result<()> {
    let raw = "credentials = { token = \"private-token\" }\n";
    let parsed: toml::Value = toml::from_str(raw)?;
    assert_eq!(
        parsed.get("credentials").and_then(|row| row.get("token")).and_then(toml::Value::as_str),
        Some("private-token")
    );
    let error = validate_hpc_campaign_config_source(raw)
        .err()
        .ok_or_else(|| anyhow::anyhow!("nested confidential token was admitted"))?;
    assert!(error.to_string().contains("sensitive key `token`"), "{error}");
    assert!(!error.to_string().contains("private-token"));
    Ok(())
}

#[test]
fn campaign_confidentiality_preserves_bare_refusal_and_reference_controls() -> anyhow::Result<()> {
    refuse_slurm("[slurm]\naccount = \"private-account\"\n")?;
    for raw in [
        "[slurm]\naccount = \"\"\n",
        "[slurm]\naccount = \"<account>\"\n",
        "[slurm]\naccount = \"${BIJUX_SLURM_ACCOUNT}\"\n",
        "token = \"\"\n",
        "secret_provider = \"file\"\nsecret_ref = \"private-reference\"\nsecret_env_key = \"PRIVATE_KEY\"\nencryption_recipients = [\"public-recipient\"]\n",
        "[secrets]\nsecret_ref = \"private-reference\"\n",
    ] {
        let _: toml::Value = toml::from_str(raw)?;
        validate_hpc_campaign_config_source(raw)?;
    }
    let error = validate_hpc_campaign_config_source("token = \"private-token\"\n")
        .err()
        .ok_or_else(|| anyhow::anyhow!("bare confidential token was admitted"))?;
    assert!(error.to_string().contains("sensitive key `token`"), "{error}");
    Ok(())
}

#[test]
fn campaign_confidentiality_checks_array_members_and_decoded_private_keys() -> anyhow::Result<()> {
    for raw in [
        "token = [\"${TOKEN}\", \"private-token\"]\n",
        "credentials = [{ token = \"private-token\" }]\n",
        "token = { value = \"private-token\" }\n",
        "token = 42\n",
        "[slurm]\naccount = [\"<account>\", \"private-account\"]\n",
    ] {
        let _: toml::Value = toml::from_str(raw)?;
        assert!(validate_hpc_campaign_config_source(raw).is_err());
    }
    for raw in ["token = []\n", "token = [\"\", \"${TOKEN}\"]\n"] {
        validate_hpc_campaign_config_source(raw)?;
    }
    let raw = "note = \"BEGIN PRIVATE\\u0020KEY\"\n";
    let error = validate_hpc_campaign_config_source(raw)
        .err()
        .ok_or_else(|| anyhow::anyhow!("decoded private key marker was admitted"))?;
    assert!(error.to_string().contains("embeds private key material"));
    assert!(!error.to_string().contains("BEGIN PRIVATE KEY"));
    Ok(())
}

#[test]
fn campaign_confidentiality_refuses_malformed_source_without_value_diagnostics() {
    for raw in [
        "token = \"private-token\n",
        "token = \"private-token\"\ntoken = \"other-private-token\"\n",
    ] {
        match validate_hpc_campaign_config_source(raw) {
            Ok(()) => panic!("malformed confidential source was admitted"),
            Err(error) => {
                assert_eq!(
                    error.to_string(),
                    "confidential config validation failed: malformed TOML source"
                );
                assert!(!format!("{error:#}").contains("private-token"));
            }
        }
    }
}
