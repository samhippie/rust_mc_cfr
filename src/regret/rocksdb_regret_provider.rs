use rocksdb::{DB, Options};
use std::error;
use std::sync::Arc;
use std::mem::transmute;
use bytevec::{ByteEncodable, ByteDecodable};

use crate::game::{Player};
use crate::regret::regret_provider::*;

pub struct RocksDbRegretProvider {
    dbs: (Arc<DB>, Arc<DB>)
}

impl RocksDbRegretProvider {
    pub fn new(name: &str) -> RocksDbRegretProvider {

        RocksDbRegretProvider {
            dbs: (
                get_db(&format!("/home/sam/data-ssd/{}{}", name, ".p1_db")), 
                get_db(&format!("/home/sam/data-ssd/{}{}", name, ".p2_db")),
            ),
        } 
    }
}

fn get_db(path: &str) -> Arc<DB> {

    let mut options = Options::default();
    options.create_if_missing(true);
    //this defaults to 64MiB, so this is a little crazy
    options.set_write_buffer_size(3 * 1024 * 1024 * 1024);
    //TODO figure out some options
    let db = DB::open(&options, path).expect("Failed to open db");
    Arc::new(db)

    //these are the options for sled
    /*
    let config = ConfigBuilder::default()
        .path(path)
        .cache_capacity(1_000_000_000)//TODO figure out a good value
        .flush_every_ms(Some(30_000))
        .build();
    */
}

impl RegretProvider for RocksDbRegretProvider {
    type Handler = RocksDbRegretHandler;

    fn get_handler(&mut self) -> RocksDbRegretHandler {
        RocksDbRegretHandler {
            dbs: (self.dbs.0.clone(), self.dbs.1.clone())
        } 
    }

    fn run(&mut self) {
        //intentionally left blank
    }
}

pub struct RocksDbRegretHandler {
    dbs: (Arc<DB>, Arc<DB>)
}

impl RegretHandler for RocksDbRegretHandler {
    
    //for these two functions, I use expect() a lot instead of handling errors
    //I'd honestly rather just crash with a message  than quietly stop
    
    fn get_regret(&self, player: Player, infoset_hash: u64) -> Result<Response, Box<dyn error::Error>> {
        let db = player.lens(&self.dbs);
        //try to get value from db
        let hash_bytes: [u8; 8] = unsafe { transmute(infoset_hash) };
        let raw = db.get(hash_bytes).expect("failed to read from db");

        let regret = if let Some(raw) = raw {
            //try to decode value
            let regret = <Vec<f32>>::decode::<u8>(&raw).expect("Failed to decode Vec<f32>");
            Some(regret)
        } else {
            None
        };

        Ok(Response::Regret(RegretResponse { regret }))

    }

    fn send_delta(&self, player: Player, infoset_hash: u64, regret_delta: Vec<f32>, iteration: i32) -> Result<(), Box<dyn error::Error>> {

        //nothing to do
        if regret_delta.len() < 2 {
            return Ok(());
        }

        let db = player.lens(&self.dbs);

        //try to get value from db
        let hash_bytes: [u8; 8] = unsafe { transmute(infoset_hash) };
        let raw = db.get(hash_bytes).expect("failed to read from db");

        let regrets_to_insert = if let Some(raw) = raw {
            //try to decode value
            let regret = <Vec<f32>>::decode::<u8>(&raw).expect("Failed to decode Vec<f32>");
            //update regrets
            let i = iteration as f32;
            regret.into_iter().zip(regret_delta.into_iter()).map(|(r,d)| {
                let new_r = r * i / (i + 1.0) + d;
                new_r.abs()
            }).collect()
        } else {
            //just insert regrets we're given
            regret_delta
        };

        let enc_regrets = regrets_to_insert.encode::<u8>().expect("failed to encode regrets");
        db.put(hash_bytes, &enc_regrets[..]).expect("failed to save regrets to db");

        Ok(())
    }
}