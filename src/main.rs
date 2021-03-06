/* -*- mode: c++; c-basic-offset: 2; indent-tabs-mode: nil; -*-
 *  vim:expandtab:shiftwidth=2:tabstop=2:smarttab:
 *
 *  Copyright (C) 2019 Red Hat, Inc.
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

/*
 * This program reads one line at a time on standard input, expecting
 * a specially formatted hostname from Apache's RewriteMap and uses
 * that to look up a build URL which it emits on standard output.
 */

#[macro_use]
extern crate custom_error;
extern crate lru;
extern crate serde_json;

use std::io::{self, BufRead};

use lru::LruCache;
use reqwest::{self, Client, Url, UrlError};
use serde_json::Value;

custom_error! {PreviewError
    InvalidData{source: UrlError} = "Garbage In",
    JsonSchema = "JSON schema problem",
    Http{source: reqwest::Error} = "HTTP Fail",
    Io{source: io::Error} = "IO Things",
}

fn main() -> Result<(), PreviewError> {
    let mut cache = LruCache::new(1024);
    let client = Client::new();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() != 2 {
            println!("Wrong number of args {:?}", parts);
            continue;
        }
        let api_url = parts[0];
        let hostname = parts[1].to_string();
        if let Some(val) = cache.get(&hostname) {
            println!("{}", val);
            continue;
        }
        let parts: Vec<&str> = hostname.split('.').collect();
        if parts.len() < 3 {
            println!("Not enough hostname parts");
            continue;
        }
        let _artifact = parts[0];
        let buildid = parts[1];
        let _tenant = parts[2];
        recoverable(|| {
            let base = Url::parse(api_url)?;
            let url = base.join(&format!("/api/build/{}", buildid))?;
            let mut response = client.get(url).send()?;
            match &response.json::<Value>()?["log_url"] {
                Value::String(log_url) => {
                    println!("{}", log_url);
                    cache.put(hostname.clone(), log_url.clone());
                    Ok(())
                }
                _ => Err(PreviewError::JsonSchema),
            }
        });
    }
    Ok(())
}

fn recoverable<F>(mut func: F)
where
    F: FnMut() -> Result<(), PreviewError>,
{
    match func() {
        Err(e) => println!("Error {}", e),
        Ok(_) => (),
    }
}
