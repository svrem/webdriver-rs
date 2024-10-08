use serde_json::json;

use crate::requests::Method;

pub struct Driver {
    port: u16,
    session_id: Option<String>,
}

pub struct Element {
    pub element_id: String,
}

fn sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

impl Driver {
    fn send_request(
        &self,
        method: Method,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, ()> {
        let session_id = match &self.session_id {
            Some(session_id) => session_id,
            None => return Err(()),
        };

        let formatted_path = format!("/session/{}{}", session_id, path);
        crate::requests::send_request(method, ("127.0.0.1", self.port), &formatted_path, body)
    }

    pub fn new(port: u16) -> Result<Driver, &'static str> {
        let res = crate::requests::send_request(Method::GET, ("127.0.0.1", port), "/", json!({}));

        if res.is_err() {
            return Err("Failed to connect to driver");
        }

        Ok(Driver {
            port,
            session_id: None,
        })
    }

    pub fn new_session(&mut self, headless: bool) -> Result<(), &'static str> {
        let body = json!({
            "capabilities": {
                "firstMatch": [{
                    "browserName": "firefox",
                    "moz:firefoxOptions": {
                        "args": if headless { vec!["-headless"] } else { vec![] }
                    }
                },
                {
                    "browserName": "chrome",
                    "goog:chromeOptions": {
                        "args": if headless { vec!["--headless"] } else { vec![] }
                    }
                }]
            }
        });

        let res = match crate::requests::send_request(
            Method::POST,
            ("127.0.0.1", self.port),
            "/session",
            body,
        ) {
            Ok(res) => res,
            Err(_) => return Err("Failed to create new session"),
        };

        if let Some(session_id) = res["value"]["sessionId"].as_str() {
            println!("Session ID: {}", session_id);
            self.session_id = Some(session_id.to_string());
            return Ok(());
        } else {
            return Err("Failed to get session ID from response");
        };
    }

    pub fn navigate_to(&self, url: &str) -> Result<(), &'static str> {
        let res = self.send_request(
            Method::POST,
            "/url",
            json!({
                "url": url
            }),
        );

        if res.is_err() {
            return Err("Failed to navigate to url");
        }

        Ok(())
    }

    pub fn find_element_by_css_selector_with_retries(
        &self,
        selector: &str,
        retries: u8,
    ) -> Result<Element, &'static str> {
        for _ in 0..retries {
            let res = self.find_element_by_css_selector(selector);
            if res.is_ok() {
                return res;
            }
            sleep(100);
        }

        Err("Failed to find element")
    }

    pub fn get_current_url(&self) -> Result<String, &'static str> {
        let res = match self.send_request(Method::GET, "/url", json!({})) {
            Ok(res) => res,
            Err(_) => return Err("Failed to get current url"),
        };

        match res["value"].as_str() {
            Some(url) => Ok(url.to_string()),
            None => Err("Failed to get current url"),
        }
    }

    pub fn find_element_by_css_selector(&self, selector: &str) -> Result<Element, &'static str> {
        let res = self.send_request(
            Method::POST,
            "/element",
            json!({
                "using": "css selector",
                "value": selector
            }),
        );

        let json_res = match res {
            Ok(res) => res,
            Err(_) => return Err("Failed to parse response"),
        };

        if json_res["value"]["message"].is_string() {
            return Err("Failed to find element");
        }

        if json_res["value"]["error"].is_string() {
            return Err("Failed to find element");
        }

        let elements = json_res["value"].as_object().unwrap();

        let key = elements.keys().next().unwrap();
        let value = elements[key].as_str().unwrap();

        Ok(Element {
            element_id: value.to_string(),
        })
    }

    pub fn click_element(&self, element: Element) -> Result<(), &'static str> {
        let res = self.send_request(
            Method::POST,
            &format!("/element/{}/click", element.element_id),
            json!({}),
        );

        if res.is_err() {
            return Err("Failed to click element");
        }

        println!("{:?}", res);

        Ok(())
    }

    pub fn send_keys(&self, element: Element, keys: &str) -> Result<(), &'static str> {
        let res = self.send_request(
            Method::POST,
            &format!("/element/{}/value", element.element_id),
            json!({
                "text": keys,
            }),
        );

        if res.is_err() {
            return Err("Failed to send keys");
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), &'static str> {
        let res = self.send_request(Method::DELETE, "", json!({})).ok();
        self.session_id = None;

        if res.is_none() {
            return Err("Failed to close session");
        }

        Ok(())
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        self.close().ok();
    }
}
