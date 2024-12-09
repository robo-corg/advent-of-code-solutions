use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Read};

type Page = u8;


#[derive(Debug)]
struct Rules {
    after: HashMap<Page, HashSet<Page>>,
    before: HashMap<Page, HashSet<Page>>,
}

impl Rules {
    fn is_update_valid(&self, update: &[Page]) -> bool {
        let mut seen = HashSet::new();
        let empty: HashSet<Page> = HashSet::new();
        let update_pages: HashSet<Page> = update.iter().copied().collect();

        for page in update.iter().copied() {
            let required_pages: HashSet<Page> = self
                .before
                .get(&page)
                .unwrap_or(&empty)
                .intersection(&update_pages).copied().collect();

            let unmet_requirements: Vec<Page> = required_pages.difference(&seen).copied().collect();

            //dbg!(page, &required_pages, &unmet_requirements);

            if !unmet_requirements.is_empty() {
                return false;
            }

            seen.insert(page);
        }

        true
    }

    fn compare(&self, a: Page, b: Page) -> Ordering {
        if let Some(afters) = self.after.get(&a) {
            if afters.contains(&b) {
                return Ordering::Less;
            }
        }

        if let Some(befores) = self.before.get(&a) {
            if befores.contains(&b) {
                return Ordering::Greater;
            }
        }

        Ordering::Equal
    }

    fn sort_update(&self, update: &[Page]) -> Vec<Page> {
        let mut sort_update = update.to_vec();

        sort_update.sort_by(|a, b| self.compare(*a, *b));

        assert!(self.is_update_valid(&sort_update));

        sort_update
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();

    let mut after: HashMap<Page, HashSet<Page>> = HashMap::new();
    let mut before: HashMap<Page, HashSet<Page>> = HashMap::new();
    let mut updates: Vec<Vec<u8>> = Vec::new();

    for maybe_line in stdin_lock.by_ref().lines() {
        let line = maybe_line?;

        if line == "" {
            break;
        }

        let (lhs_s, rhs_s) = line.split_once('|').unwrap();

        let lhs = u8::from_str_radix(lhs_s, 10)?;
        let rhs = u8::from_str_radix(rhs_s, 10)?;

        //rules.insert(lhs, rhs);
        after.entry(lhs).or_default().insert(rhs);
        before.entry(rhs).or_default().insert(lhs);
    }

    let rules = Rules { after, before };

    for maybe_line in stdin_lock.lines() {
        let line = maybe_line?;

        let update = line
            .split(',')
            .map(|p_s| u8::from_str_radix(p_s, 10).unwrap())
            .collect();
        updates.push(update);
    }

    dbg!(&rules);
    dbg!(&updates);

    let mut total = 0;

    for update in updates.iter() {
        if rules.is_update_valid(&update) {
            println!("{:?} is valid", update);

            total += update[update.len()/2] as i32;
        }
        else {
            println!("{:?} is invalid", update);
        }
    }

    println!("total(part1): {}", total);


    let mut total = 0;

    for update in updates.iter() {
        if !rules.is_update_valid(&update) {
            let sorted_update = rules.sort_update(update);

            total += sorted_update[sorted_update.len()/2] as i32;
        }
    }


    println!("total(part2): {}", total);


    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_one() {}
}
