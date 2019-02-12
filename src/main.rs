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

extern crate lru;
extern crate serde_json;
extern crate reqwest;

use std::io::{self, Read};

use serde_json::Value;
use lru::LruCache;
use reqwest::{Client,Url};


fn main() -> io::Result<()> {
    let mut cache = LruCache::new(1024);
    let mut buffer = String::new();
    let mut client = Client::new();
    loop {
        io::stdin().read_to_string(&mut buffer)?;
        let parts: Vec<&str> = buffer.split(' ').collect();
        if parts.len() != 2 {
            println!("Wrong number of args");
            continue
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
            continue
        }
        let artifact = parts[0];
        let buildid = parts[1];
        let tenant = parts[2];
        let base = Url::parse(api_url).unwrap();
        let url = base.join(format!("{}/build/{}", &tenant, &buildid)).unwrap();
        let response = client.get(url).send().unwrap();
        let body: Value = response.json().unwrap();
        println!("{}", body["log_url"]);
        cache.put(hostname, body["log_url"]);
    }
}
