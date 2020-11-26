use super::*;

#[derive(Serialize, Deserialize)]
struct RawGroupings(Vec<RawGrouping>);
#[derive(Serialize, Deserialize)]
struct RawGrouping {
    name: String,
    validators: Vec<PublicKey>,
}
impl<'fbas> Groupings<'fbas> {
    pub fn from_json_str(json: &str, fbas: &'fbas Fbas) -> Self {
        Self::from_raw(
            serde_json::from_str(json).expect("Error parsing Organizations JSON"),
            fbas,
        )
    }
    pub fn from_json_file(path: &Path, fbas: &'fbas Fbas) -> Self {
        let json =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Error reading file {:?}", path));
        Self::from_json_str(&json, fbas)
    }
    fn from_raw(raw_groupings: RawGroupings, fbas: &'fbas Fbas) -> Self {
        let groupings: Vec<Grouping> = raw_groupings
            .0
            .into_iter()
            .map(|x| Grouping::from_raw(x, &fbas.pk_to_id))
            .collect();

        Groupings::new(groupings, fbas)
    }
    fn to_raw(&self) -> RawGroupings {
        RawGroupings(
            self.groupings
                .iter()
                .map(|org| org.to_raw(self.fbas))
                .collect(),
        )
    }
    pub fn load_isps_from_file(path: &Path, fbas: &'fbas Fbas) -> Self {
        let json =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Error reading file {:?}", path));
        Self::load_isps_from_str(&json, &fbas)
    }
    pub fn load_isps_from_str(json: &str, fbas: &'fbas Fbas) -> Self {
        let raw_nodes: Vec<RawNode> = serde_json::from_str(&json).expect("Error parsing FBAS JSON");
        let raw_groupings = Groupings::get_isps_from_raw_nodes(raw_nodes);
        Groupings::from_raw(raw_groupings, &fbas)
    }
    pub fn load_countries_from_file(path: &Path, fbas: &'fbas Fbas) -> Self {
        let json =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Error reading file {:?}", path));
        Self::load_countries_from_str(&json, &fbas)
    }
    pub fn load_countries_from_str(json: &str, fbas: &'fbas Fbas) -> Self {
        let raw_nodes: Vec<RawNode> = serde_json::from_str(&json).expect("Error parsing FBAS JSON");
        let raw_groupings = Groupings::get_countries_from_raw_nodes(raw_nodes);
        Groupings::from_raw(raw_groupings, &fbas)
    }
    fn get_isps_from_raw_nodes(raw_nodes: Vec<RawNode>) -> RawGroupings {
        let mut isp_to_validators: HashMap<String, Vec<PublicKey>> =
            HashMap::with_capacity(raw_nodes.len());
        let mut raw_groupings: Vec<RawGrouping> = Vec::with_capacity(isp_to_validators.len());
        for raw_node in &raw_nodes {
            if let Some(name) = &raw_node.isp {
                let mut isp = name.clone();
                isp = Groupings::remove_special_chars_from_grouping_name(isp);
                if isp_to_validators.get(&isp) == None {
                    isp_to_validators.insert(isp.clone(), Vec::new());
                }
                isp_to_validators
                    .get_mut(&isp)
                    .unwrap()
                    .push(raw_node.public_key.clone());
            };
        }
        let mut grouping_names = Vec::with_capacity(isp_to_validators.len());
        for key in isp_to_validators.keys() {
            grouping_names.push(key);
        }
        grouping_names.sort();
        for name in grouping_names {
            if let Some(validators) = isp_to_validators.get(name) {
                let raw_grouping = RawGrouping {
                    name: name.clone(),
                    validators: validators.clone(),
                };
                raw_groupings.push(raw_grouping);
            }
        }
        RawGroupings(raw_groupings)
    }
    fn get_countries_from_raw_nodes(raw_nodes: Vec<RawNode>) -> RawGroupings {
        let mut country_to_validators: HashMap<String, Vec<PublicKey>> =
            HashMap::with_capacity(raw_nodes.len());
        let mut raw_groupings: Vec<RawGrouping> = Vec::with_capacity(country_to_validators.len());
        for raw_node in &raw_nodes {
            if let Some(geodata) = &raw_node.geo_data {
                if let Some(name) = &geodata.country_name {
                    let mut country = name.clone();
                    country = Groupings::remove_special_chars_from_grouping_name(country);
                    if country_to_validators.get(&country.clone()) == None {
                        country_to_validators.insert(country.clone(), Vec::new());
                    }
                    country_to_validators
                        .get_mut(&country.clone())
                        .unwrap()
                        .push(raw_node.public_key.clone());
                }
            };
        }
        let mut grouping_names = Vec::with_capacity(country_to_validators.len());
        for key in country_to_validators.keys() {
            grouping_names.push(key);
        }
        grouping_names.sort();
        for name in grouping_names {
            if let Some(validators) = country_to_validators.get(name) {
                let raw_grouping = RawGrouping {
                    name: name.clone(),
                    validators: validators.clone(),
                };
                raw_groupings.push(raw_grouping);
            }
        }
        RawGroupings(raw_groupings)
    }
    fn remove_special_chars_from_grouping_name(mut name: String) -> String {
        name.retain(|c| c != ',');
        let mut maybe_fullstop = name.split_off(name.len() - 1);
        maybe_fullstop.retain(|c| c != '.');
        name.push_str(&maybe_fullstop);
        name
    }
}
impl<'fbas> Serialize for Groupings<'fbas> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_raw().serialize(serializer)
    }
}
impl Grouping {
    fn from_raw(raw_grouping: RawGrouping, pk_to_id: &HashMap<PublicKey, NodeId>) -> Self {
        Grouping {
            name: raw_grouping.name,
            validators: raw_grouping
                .validators
                .into_iter()
                .filter_map(|pk| pk_to_id.get(&pk))
                .cloned()
                .collect(),
        }
    }
    fn to_raw(&self, fbas: &Fbas) -> RawGrouping {
        RawGrouping {
            name: self.name.clone(),
            validators: self
                .validators
                .iter()
                .map(|&x| fbas.nodes[x].public_key.clone())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_isps_from_nodes_json_str() {
        let json = r#"[
            {
                "publicKey": "GCGB2",
                "active": true,
                "isp": "Google.com"
            },
            {
                "publicKey": "GCM6Q",
                "active": true,
                "isp": "StackOverflow"
            },
            {
                "publicKey": "GCHAR",
                "active": true,
                "isp": "Hetzner"
            },
            {
                "publicKey": "GABMK",
                "active": true,
                "isp": "Google.com"
            }]"#;
        let fbas = Fbas::from_json_str(&json);
        let isps = Groupings::load_isps_from_str(&json, &fbas);
        let expected_names = vec!["Google.com", "Hetzner", "StackOverflow"];
        let actual_names: Vec<String> = isps.groupings.iter().map(|x| x.name.clone()).collect();
        let expected_validators: Vec<Vec<NodeId>> = vec![vec![0, 3], vec![2], vec![1]];
        let actual_validators: Vec<Vec<NodeId>> = isps
            .groupings
            .iter()
            .map(|x| x.validators.clone())
            .collect();
        assert_eq!(expected_names, actual_names);
        assert_eq!(expected_validators, actual_validators);
    }
    #[test]
    fn read_countries_from_nodes_json_str() {
        let json = r#"[
            {
                "publicKey": "GCGB2",
                "active": true,
                "geoData": {
                    "countryCode": "AA",
                    "countryName": "Absurdistan"
                }
            },
            {
                "publicKey": "GCM6Q",
                "active": true,
                "geoData": {
                    "countryCode": "WA",
                    "countryName": "Wakanda"
                }
            },
            {
                "publicKey": "GCHAR",
                "active": true,
                "geoData": {
                    "countryCode": "TI",
                    "countryName": "Timbuktu"
                }
            },
            {
                "publicKey": "GABMK",
                "active": true,
                "geoData": {
                    "countryCode": "TI",
                    "countryName": "Timbuktu"
                }
            }]"#;
        let fbas = Fbas::from_json_str(&json);
        let countries = Groupings::load_countries_from_str(&json, &fbas);
        let expected_names = vec!["Absurdistan", "Timbuktu", "Wakanda"];
        let actual_names: Vec<String> =
            countries.groupings.iter().map(|x| x.name.clone()).collect();
        let expected_validators: Vec<Vec<NodeId>> = vec![vec![0], vec![2, 3], vec![1]];
        let actual_validators: Vec<Vec<NodeId>> = countries
            .groupings
            .iter()
            .map(|x| x.validators.clone())
            .collect();
        assert_eq!(expected_names, actual_names);
        assert_eq!(expected_validators, actual_validators);
    }
    #[test]
    fn missing_ctry_key_in_json_doesnt_panic() {
        let json = r#"[
            {
                "publicKey": "GCGB2",
                "geoData": {
                    "countryName": "Wakanda"
                }
            },
            {
                "publicKey": "GCM6Q",
                "geoData": {
                    "countryName": "Absurdistan"
                }
            },
            {
                "publicKey": "GABMK"
            }]"#;
        let fbas = Fbas::from_json_str(&json);
        let countries = Groupings::load_countries_from_str(&json, &fbas);
        let expected_names = vec!["Absurdistan", "Wakanda"];
        let actual_names: Vec<String> =
            countries.groupings.iter().map(|x| x.name.clone()).collect();
        let expected_validators: Vec<Vec<NodeId>> = vec![vec![1], vec![0]];
        let actual_validators: Vec<Vec<NodeId>> = countries
            .groupings
            .iter()
            .map(|x| x.validators.clone())
            .collect();
        assert_eq!(expected_names, actual_names);
        assert_eq!(expected_validators, actual_validators);
    }
    #[test]
    fn special_chars_filtered_from_json_str() {
        let json = r#"[
            {
                "publicKey": "GCGB2",
                "isp": "Google.com"
            },
            {
                "publicKey": "GCM6Q",
                "isp": "Google.com."
            },
            {
                "publicKey": "GCHAR",
                "isp": "Amazon.com Inc,"
            },
            {
                "publicKey": "GCARK",
                "isp": "Google.com,"
            },
            {
                "publicKey": "GABMK",
                "isp": "Amazon.com, Inc."
            }]"#;
        let fbas = Fbas::from_json_str(&json);
        let isps = Groupings::load_isps_from_str(&json, &fbas);
        let expected_names = vec!["Amazon.com Inc", "Google.com"];
        let actual_names: Vec<String> = isps.groupings.iter().map(|x| x.name.clone()).collect();
        let expected_validators: Vec<Vec<NodeId>> = vec![vec![2, 4], vec![0, 1, 3]];
        let actual_validators: Vec<Vec<NodeId>> = isps
            .groupings
            .iter()
            .map(|x| x.validators.clone())
            .collect();
        assert_eq!(expected_names, actual_names);
        assert_eq!(expected_validators, actual_validators);
    }
}
