pub(crate) enum Case {
    FourXs,
}

pub(crate) fn four_xs(
    name: &str,
    val: serde_json::Value,
) -> crate::GenericResult<()> {
    crate::quote_value(name, val)?;
    Ok(())
}
