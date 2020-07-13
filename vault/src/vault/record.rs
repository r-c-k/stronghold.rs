use crate::{
    types::{
        transactions::{DataTransaction, InitTransaction, RevocationTransaction},
        utils::Id,
    },
    vault::entries::Entry,
};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// Record over all entries by an owner and ordered by the counter
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainRecord(HashMap<Id, Vec<Entry>>);

// Record of all valid entries.
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidRecord(HashMap<Id, Entry>);

impl ChainRecord {
    // create a new record
    pub fn new(i: impl Iterator<Item = Entry>) -> crate::Result<Self> {
        // sort entries by owner
        let mut chains: HashMap<_, Vec<Entry>> = HashMap::new();
        i.for_each(|e| chains.entry(e.owner()).or_default().push(e.clone()));

        // order chains and remove all non-referenced transactions
        for (_, chain) in chains.iter_mut() {
            // sort transactions by counter
            chain.sort_by_key(|e| e.ctr());
            let (start, mut ctr) = chain
                .iter()
                .enumerate()
                .rev()
                .find_map(|(start, e)| Some((start, e.typed::<InitTransaction>()?.ctr)))
                .ok_or(crate::Error::ChainError(String::from(
                    "Chain does not contain a start transaction",
                )))?;

            // get transactions that are ancestors of the InitTransaction
            *chain = chain
                .iter()
                .skip(start)
                .take_while(|e| e.ctr() == ctr.postfix_increment())
                .cloned()
                .collect();
        }
        Ok(ChainRecord(chains))
    }

    // get chains by owner
    pub fn owners(&self) -> impl Iterator<Item = (&Id, &[Entry])> {
        self.0.iter().map(|(id, chain)| (id, chain.as_slice()))
    }

    // get an entry in the chain by owner
    pub fn get(&self, owner: &Id) -> Option<&[Entry]> {
        self.0.get(owner).map(|e| e.as_slice())
    }

    // get all entries owned by the owner or panic
    pub fn force_get(&self, owner: &Id) -> &[Entry] {
        self.get(owner).expect("There is no chain for this owner")
    }

    // get the last entry of a chain by owner
    pub fn force_last(&self, owner: &Id) -> &Entry {
        self.force_get(owner)
            .last()
            .expect("The chain is empty and thus has no last entry")
    }

    // get all entries
    pub fn all(&self) -> impl Iterator<Item = &Entry> {
        self.0.values().flatten()
    }

    // get all revoked transactions in the chain by owner
    pub fn own_revoked(&self, owner: &Id) -> impl Iterator<Item = (Id, &Entry)> {
        let chain = self.force_get(owner);
        chain
            .iter()
            .filter_map(|e| Some((e.typed::<RevocationTransaction>()?.id, e)))
    }

    // get all foreign data not owned by the id
    pub fn foreign_data(&self, except: &Id) -> impl Iterator<Item = &Entry> {
        let except = *except;
        self.0
            .iter()
            .filter(move |(owner, _)| **owner != except)
            .map(|(_, chain)| chain)
            .flatten()
            .filter(|e| e.typed::<DataTransaction>().is_some())
    }
}

impl ValidRecord {
    // create a new valid record
    pub fn new(chains: &ChainRecord) -> Self {
        // collect the data and remove revoked ones
        let mut valid: HashMap<_, _> = chains
            .all()
            .filter_map(|e| Some((e.typed::<DataTransaction>()?.id, e.clone())))
            .collect();
        chains
            .all()
            .filter_map(|e| e.typed::<RevocationTransaction>())
            .for_each(|r| {
                valid.remove(&r.id);
            });

        // shrink the map
        valid.shrink_to_fit();
        Self(valid)
    }

    // get chain by id
    pub fn get(&self, id: &Id) -> Option<&Entry> {
        self.0.get(id)
    }

    // get all valid entries
    pub fn all(&self) -> impl Iterator<Item = &Entry> + ExactSizeIterator {
        self.0.values()
    }

    // get all valid for owner id
    pub fn all_for_owner(&self, owner: &Id) -> impl Iterator<Item = &Entry> {
        let owner = *owner;
        self.all().filter(move |e| e.owner() == owner)
    }
}