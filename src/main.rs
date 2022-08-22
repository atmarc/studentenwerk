use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;
use serde::Deserialize;
use std::collections::{HashSet, HashMap};


static BASE_URL: &str = "https://www.studentenwerk-muenchen.de";
static OFFERS_URL: &str = "/en/accommodation/private-accommodation-service/offers/";
static CACHE_PATH: &str = "offers_cache.json";

#[derive(Debug, Deserialize)]
struct Offer {
    id: String,
    link: String,
    address: String,
    room_type: String,
    cost: String,
    n_rooms: String,
    size: String
}

impl Offer {
    fn new(array: Vec<String>) -> Offer {
        Offer {
            id: array[0].to_owned(),
            link: array[1].to_owned(),
            address: array[2].to_owned(),
            room_type: array[3].to_owned(),
            cost: array[4].to_owned(),
            n_rooms: array[5].to_owned(),
            size: array[6].to_owned()  
        }
    }

    fn to_json (&self) -> String {
        format!(
"    {{
        \"id\":\"{}\",
        \"link\":\"{}\",
        \"address\":\"{}\",
        \"room_type\":\"{}\",
        \"cost\":\"{}\",
        \"n_rooms\":\"{}\",
        \"size\":\"{}\"
    }}", 
            self.id, 
            self.link, 
            self.address, 
            self.room_type, 
            self.cost, 
            self.n_rooms, 
            self.size
        )
    }
}

fn get_offers() -> Vec<Offer> {
    let url = [BASE_URL, OFFERS_URL].concat();
    let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    
    let document = Html::parse_document(&body);
    let selector_tbody = Selector::parse("tbody").unwrap();
    let selector_tr = Selector::parse("tr").unwrap();
    let selector_td = Selector::parse("td").unwrap();
    let selector_a = Selector::parse("a").unwrap();

    let first_table = document.select(&selector_tbody).next().unwrap();
    
    let mut data = Vec::new();

    for line in first_table.select(&selector_tr) {
        let mut props = line.select(&selector_td);
        let a_elem = props.next().unwrap().select(&selector_a)
            .next().unwrap();
        
            let url = a_elem.value().attr("href").unwrap();
        let id = a_elem.inner_html();
        let address = props.next().unwrap().inner_html();
        let room_type = props.next().unwrap().inner_html();
        let cost = props.next().unwrap().inner_html();
        let n_rooms = props.next().unwrap().inner_html();
        let size = props.next().unwrap().inner_html();
        
        let mut offer_data = 
            vec![id, url.to_string(), address, room_type, cost, n_rooms, size];
        
        offer_data = offer_data.iter().map(
            |v| v.replace("\t", "").replace("\n", "")
                .replace("<br>", " ")
        ).collect();

        let new_offer = Offer::new(offer_data);

        data.push(new_offer);
    }
    data
}

fn store_data(data: &Vec<Offer>) {
    let offers_jsons: Vec<String> = 
        data.iter().map(|o| o.to_json()).collect();
    
    let output = format!("[\n{}\n]", offers_jsons.join(",\n"));

    let mut file = File::create(CACHE_PATH)
        .expect("Can't open the file");
    
    file.write_all(output.as_bytes()).expect("Error while writting");
}

fn read_stored_data() -> Vec<Offer> {
    match File::open(CACHE_PATH) {
        Ok(res) => {
            println!("Offers found on {}!", CACHE_PATH);
            serde_json::from_reader(res).expect("Error parsing JSON")
        },
        _ => Vec::new(),  
    }
}

fn main() {
    println!("Starting program!");

    println!("Getting new offers...");
    let today_offers = get_offers();

    println!("Looking for stored offers...");
    let previous_offers = read_stored_data();

    let today_ids: HashSet<String> = 
        today_offers.iter().map(|v| v.id.clone()).collect();

    let previous_ids: HashSet<String> = 
        previous_offers.iter().map(|v| v.id.clone()).collect();

    let mut all_data: HashMap<String, &Offer> = HashMap::new(); 
    let mut new_ids: Vec<String> = Vec::new();

    // look for new offers    
    for i in 0..today_offers.len() {
        let tmp_offer = &today_offers[i];

        if !previous_ids.contains(&tmp_offer.id) {
            new_ids.push(tmp_offer.id.clone());
        }

        all_data.insert(tmp_offer.id.clone(), &today_offers[i]);
    }
    
    // Look for removed offers
    let mut removed_ids: Vec<String> = Vec::new();
    for i in 0..previous_offers.len() {
        let tmp_offer = &previous_offers[i];

        if !today_ids.contains(&tmp_offer.id) {
            removed_ids.push(tmp_offer.id.clone());
        }

        if !all_data.contains_key(&tmp_offer.id) {
            all_data.insert(tmp_offer.id.clone(), &previous_offers[i]);
        }
    }

    println!("New Ids: {:?}", new_ids);
    println!("Removed Ids: {:?}", removed_ids);
    
    println!("Storing new values...");
    store_data(&today_offers);
}
