extern crate reqwest;
extern crate scraper;
use scraper::{Html, Selector};
use selectors::attr::CaseSensitivity;
use std::fmt;

#[derive(Debug)]
struct OffererStruct {
    id: String,
    name_store: String,
    address: String,
    accept_credit_card: bool,
    accept_delivery: bool,
    price: String,
}

#[derive(Debug)]
struct ProductStruct {
    id: String,
    specifications: String,
    model: String,
    price: String,
    image: String,
    number_of_stores: String,
    offerers: Vec<OffererStruct>,
}

impl fmt::Display for OffererStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Loja: {} \r
            ",
            self.name_store
        )
    }
}

impl fmt::Display for ProductStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Produto: {} \r
            Especificação: {} \r
            Menor preço: {} \r
            Encontrado: {} \r
            Ofertantes: {:?} \r
            \n
            ",
            self.model, self.specifications, self.price, self.number_of_stores, self.offerers
        )
    }
}

enum TypeQuery {
    HREF,
    SRC,
    TEXT,
    HasClass,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    scrape_data("https://www.boadica.com.br/busca-resultado?q=caneta").await?;
    Ok(())
}

async fn scrape_data_salles(mut product: ProductStruct) -> Result<(), reqwest::Error> {
    let url_base = String::from("https://www.boadica.com.br/produtos/") + &product.id;

    let req = reqwest::get(&url_base).await?;
    assert!(req.status().is_success());

    let doc_body = Html::parse_document(&req.text().await?);
    let selector = Selector::parse("#precos .row").unwrap();

    for element in doc_body.select(&selector) {
        let doc_body = Html::parse_document(&element.html());
        let id: String = get_by(&doc_body, String::from("a"), TypeQuery::HREF)
            .split("/loja/")
            .collect();

        let name_store: String = get_by(&doc_body, String::from("a"), TypeQuery::TEXT);
        let address: String = get_by(
            &doc_body,
            String::from("div:nth-child(1) > div.col-md-6"),
            TypeQuery::TEXT,
        );

        let accept_credit_card = get_by(
            &doc_body,
            String::from(".fa-credit-card"),
            TypeQuery::HasClass,
        );

        let accept_delivery = get_by(
            &doc_body,
            String::from(".fa-motorcycle"),
            TypeQuery::HasClass,
        );

        let price = get_by(&doc_body, String::from(".preco-loja"), TypeQuery::TEXT);

        let offer = OffererStruct {
            id: (&id.trim()).to_string(),
            name_store: name_store,
            address: address,
            accept_credit_card: accept_credit_card == "true",
            accept_delivery: accept_delivery == "true",
            price: price,
        };

        product.offerers.push(offer);
    }

    println!("prod {}", product);
    Ok(())
}

async fn scrape_data(url: &str) -> Result<(), reqwest::Error> {
    let req = reqwest::get(url).await?;
    assert!(req.status().is_success());

    let doc_body = Html::parse_document(&req.text().await?);
    let selector = Selector::parse(".produto").unwrap();

    for element in doc_body.select(&selector) {
        let doc_body = Html::parse_document(&element.html());
        let id: String = get_by(&doc_body, String::from("a"), TypeQuery::HREF)
            .split("/produtos/")
            .collect();

        let product = ProductStruct {
            model: get_by(&doc_body, String::from(".titulo a"), TypeQuery::TEXT),
            specifications: get_by(&doc_body, String::from(".especificacao"), TypeQuery::TEXT),
            price: get_by(&doc_body, String::from(".preco strong"), TypeQuery::TEXT),
            image: get_by(&doc_body, String::from("a img"), TypeQuery::SRC),
            number_of_stores: get_by(&doc_body, String::from(".lojas"), TypeQuery::TEXT),
            id: (&id.trim()).to_string(),
            offerers: Vec::new(),
        };
        scrape_data_salles(product).await?;
        //  println!("{:?}", product);
    }
    Ok(())
}

fn get_by(doc_body: &scraper::Html, query_selector: String, type_query: TypeQuery) -> String {
    let selector = Selector::parse(&query_selector).unwrap();
    let mut content: String = "".to_string();
    let mut element: scraper::ElementRef;
    for el_specifications in doc_body.select(&selector) {
        element = el_specifications;
        content = match type_query {
            TypeQuery::SRC => element.value().attr("src").unwrap_or("").to_string(),
            TypeQuery::HREF => element.value().attr("href").unwrap_or("").to_string(),
            TypeQuery::HasClass => {
                if element
                    .value()
                    .has_class("", CaseSensitivity::CaseSensitive)
                {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            TypeQuery::TEXT => {
                let el = element.text().collect::<Vec<_>>();
                let el: Vec<_> = el
                    .iter()
                    .map(|&x| x.replace("\n", "").trim().to_string())
                    .collect::<Vec<_>>();

                let el: Vec<_> = el.iter().filter(|&x| x.len() > 0).collect::<Vec<_>>();
                let el = el.iter().max();

                match el {
                    Some(x) => x.to_string(),
                    None => "".to_string(),
                }
            }
        };
    }

    (content.trim()).to_string()
}
