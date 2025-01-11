use std::collections::{BTreeMap, VecDeque};

#[derive(Default)]
struct TrieNode {
    pub children: BTreeMap<char, usize>,
    fallback_node: usize,
    output: Vec<usize>,
}

#[derive(Default)]
pub struct Matcher {
    nodes: Vec<TrieNode>,
    current: usize,
}

impl Matcher {
    pub fn new(words: Vec<String>) -> Self {
        let mut nodes: Vec<TrieNode> = vec![];
        nodes.push(TrieNode::default());

        for (i, word) in words.iter().enumerate() {
            let mut current_node: usize = 0;
            for c in word.chars() {
                current_node = if let Some(next) = nodes[current_node].children.get(&c) {
                    *next
                } else {
                    let n: usize = nodes.len();
                    nodes[current_node].children.insert(c, n);
                    nodes.push(TrieNode::default());
                    n
                }
            }
            nodes[current_node].output.push(i);
        }

        // now we use bfs to link the fallback nodes
        let mut queue: VecDeque<usize> = vec![].into();
        // first we add the nodes of level 1;
        for node in nodes[0].children.values() {
            queue.push_back(*node);
        }

        while let Some(parent) = queue.pop_front() {
            for (&c, &child) in nodes[parent].children.clone().iter() {
                let mut fallback: usize = nodes[parent].fallback_node;
                while fallback != 0 && !nodes[fallback].children.contains_key(&c) {
                    fallback = nodes[fallback].fallback_node
                }
                if let Some(&next) = nodes[fallback].children.get(&c) {
                    fallback = next;
                }
                nodes[child].fallback_node = fallback;
                let mut sub_words: Vec<usize> = nodes[fallback].output.clone();
                nodes[child].output.append(&mut sub_words);

                queue.push_back(child);
            }
        }

        Self { nodes, current: 0 }
    }

    pub fn next(&mut self, c: &char) -> Vec<usize> {
        if let Some(&next_node) = self.nodes[self.current].children.get(c) {
            self.current = next_node;
            return self.nodes[next_node].output.clone();
        }
        if self.current == 0 {
            return vec![];
        }
        self.current = self.nodes[self.current].fallback_node;
        self.next(c)
    }

    pub fn options(&self, state: usize) -> Vec<char> {
        self.nodes[state].children.keys().cloned().collect()
    }

    pub fn next_state(&self, state: usize, c: char) -> usize {
        if let Some(&next_state) = self.nodes[state].children.get(&c) {
            return next_state;
        }
        if state == 0 {
            return 0;
        }
        self.next_state(self.nodes[state].fallback_node, c)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_general_case() {
        let words: Vec<String> = ["he", "she", "his", "hers"]
            .iter()
            .map(|&s| s.to_string())
            .collect();
        let mut matcher: Matcher = Matcher::new(words);
        let text: String = "shers".to_string();
        let ans: Vec<Vec<usize>> = vec![vec![], vec![], vec![1, 0], vec![], vec![3]];
        for (i, c) in text.chars().enumerate() {
            let sac: BTreeSet<usize> = BTreeSet::from_iter(matcher.next(&c));
            assert_eq!(sac, BTreeSet::from_iter(ans[i].clone()));
        }
    }

    #[test]
    fn test_fallback() {
        let words: Vec<String> = ["abck", "bcdk", "cde", "def", "abkcdefg"]
            .iter()
            .map(|&s| s.to_string())
            .collect();
        let mut matcher: Matcher = Matcher::new(words);
        let text: String = "abcdefg".to_string();
        let ans: Vec<Vec<usize>> = vec![vec![], vec![], vec![], vec![], vec![2], vec![3], vec![]];
        for (i, c) in text.chars().enumerate() {
            let sac: BTreeSet<usize> = BTreeSet::from_iter(matcher.next(&c));
            assert_eq!(sac, BTreeSet::from_iter(ans[i].clone()));
        }
    }

    #[test]
    fn test_no_matches() {
        let words: Vec<String> = ["xyz", "abc", "deb", "kef"]
            .iter()
            .map(|&s| s.to_string())
            .collect();
        let mut matcher: Matcher = Matcher::new(words);
        let text: String = "def".to_string();
        let ans: Vec<Vec<usize>> = vec![vec![], vec![], vec![]];
        for (i, c) in text.chars().enumerate() {
            let sac: BTreeSet<usize> = BTreeSet::from_iter(matcher.next(&c));
            assert_eq!(sac, BTreeSet::from_iter(ans[i].clone()));
        }
    }

    #[test]
    fn test_full_text_match() {
        let words: Vec<String> = ["a", "aa", "aaa"].iter().map(|&s| s.to_string()).collect();
        let mut matcher: Matcher = Matcher::new(words);
        let text: String = "aaa".to_string();
        let ans: Vec<Vec<usize>> = vec![vec![0], vec![1, 0], vec![2, 1, 0]];
        for (i, c) in text.chars().enumerate() {
            let sac: BTreeSet<usize> = BTreeSet::from_iter(matcher.next(&c));
            assert_eq!(sac, BTreeSet::from_iter(ans[i].clone()));
        }
    }
}
