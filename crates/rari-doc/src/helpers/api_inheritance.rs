use super::json_data::json_data_interface;

pub fn inheritance(main_if: &str) -> Vec<&str> {
    let web_api_data = json_data_interface();
    let mut inherited = vec![];

    let mut interface = main_if;
    while let Some(inherited_data) = web_api_data
        .get(interface)
        .map(|data| data.inh.as_str())
        .filter(|ihn| !ihn.is_empty())
    {
        inherited.push(inherited_data);
        interface = inherited_data;
    }
    inherited
}
