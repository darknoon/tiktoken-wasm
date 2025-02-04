use anyhow::{anyhow, Error};
use base64::{engine::general_purpose, Engine as _};
use fancy_regex::Regex;
use gloo_utils::format::JsValueSerdeExt;
use rustc_hash::FxHashMap as HashMap;
use std::collections::HashSet;
use std::result::Result;
use wasm_bindgen::prelude::*;

#[cfg(feature = "inline")]
const ENDOFTEXT: &'static str = "<|endoftext|>";

#[cfg(feature = "inline")]
const FIM_PREFIX: &'static str = "<|fim_prefix|>";

#[cfg(feature = "inline")]
const FIM_MIDDLE: &'static str = "<|fim_middle|>";

#[cfg(feature = "inline")]
const FIM_SUFFIX: &'static str = "<|fim_suffix|>";

#[cfg(feature = "inline")]
const ENDOFPROMPT: &'static str = "<|endofprompt|>";

struct CoreBPEConstructor {
    encoder: HashMap<Vec<u8>, usize>,
    special_tokens: HashMap<String, usize>,
    pat_str: String,
}

impl CoreBPEConstructor {
    fn new(
        tiktoken_bfe: &str,
        special_tokens: Option<HashMap<String, usize>>,
        pat_str: &str,
    ) -> Self {
        CoreBPEConstructor {
            encoder: CoreBPEConstructor::parse_bfe(tiktoken_bfe).unwrap(),
            special_tokens: special_tokens.unwrap_or_default(),
            pat_str: String::from(pat_str),
        }
    }

    fn parse_bfe(tiktoken_bfe: &str) -> Result<HashMap<Vec<u8>, usize>, Error> {
        let mut encoder = HashMap::default();
        for line in tiktoken_bfe.lines() {
            let mut parts = line.split(' ');
            let token = &general_purpose::STANDARD.decode(parts.next().unwrap())?;
            let rank: usize = parts.next().unwrap().parse().unwrap();
            encoder.insert(token.clone(), rank);
        }

        Ok(encoder)
    }

    #[cfg(feature = "inline")]
    fn gpt2() -> Self {
        let mut special_tokens = HashMap::default();
        special_tokens.insert(String::from(ENDOFTEXT), 50256);

        CoreBPEConstructor::new(
            include_str!("../ranks/gpt2.tiktoken"),
            Some(special_tokens),
            "'s|'t|'re|'ve|'m|'ll|'d| ?\\p{L}+| ?\\p{N}+| ?[^\\s\\p{L}\\p{N}]+|\\s+(?!\\S)|\\s+",
        )
    }

    #[cfg(feature = "inline")]
    fn r50k_base() -> Self {
        let mut special_tokens = HashMap::default();
        special_tokens.insert(String::from(ENDOFTEXT), 50256);

        CoreBPEConstructor::new(
            include_str!("../ranks/r50k_base.tiktoken"),
            Some(special_tokens),
            "'s|'t|'re|'ve|'m|'ll|'d| ?\\p{L}+| ?\\p{N}+| ?[^\\s\\p{L}\\p{N}]+|\\s+(?!\\S)|\\s+",
        )
    }

    #[cfg(feature = "inline")]
    fn p50k_base() -> Self {
        let mut special_tokens = HashMap::default();
        special_tokens.insert(String::from(ENDOFTEXT), 50256);

        CoreBPEConstructor::new(
            include_str!("../ranks/p50k_base.tiktoken"),
            Some(special_tokens),
            "'s|'t|'re|'ve|'m|'ll|'d| ?\\p{L}+| ?\\p{N}+| ?[^\\s\\p{L}\\p{N}]+|\\s+(?!\\S)|\\s+",
        )
    }

    #[cfg(feature = "inline")]
    fn p50k_edit() -> Self {
        let mut special_tokens = HashMap::default();
        special_tokens.insert(String::from(ENDOFTEXT), 50256);
        special_tokens.insert(String::from(FIM_PREFIX), 50281);
        special_tokens.insert(String::from(FIM_MIDDLE), 50282);
        special_tokens.insert(String::from(FIM_SUFFIX), 50283);

        CoreBPEConstructor::new(
            include_str!("../ranks/p50k_base.tiktoken"),
            Some(special_tokens),
            "'s|'t|'re|'ve|'m|'ll|'d| ?\\p{L}+| ?\\p{N}+| ?[^\\s\\p{L}\\p{N}]+|\\s+(?!\\S)|\\s+",
        )
    }

    #[cfg(feature = "inline")]
    fn cl100k_base() -> Self {
        let mut special_tokens = HashMap::default();
        special_tokens.insert(String::from(ENDOFTEXT), 100257);
        special_tokens.insert(String::from(FIM_PREFIX), 100258);
        special_tokens.insert(String::from(FIM_MIDDLE), 100259);
        special_tokens.insert(String::from(FIM_SUFFIX), 100260);
        special_tokens.insert(String::from(ENDOFPROMPT), 100276);

        CoreBPEConstructor::new(
            include_str!("../ranks/cl100k_base.tiktoken"),
            Some(special_tokens),
            "(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\\r\\n\\p{L}\\p{N}]?\\p{L}+|\\p{N}{1,3}| ?[^\\s\\p{L}\\p{N}]+[\\r\\n]*|\\s*[\\r\\n]+|\\s+(?!\\S)|\\s+",
        )
    }
}

#[wasm_bindgen]
pub struct Tiktoken {
    name: Option<String>,
    special_tokens_set: HashSet<String>,
    bpe: CoreBPE,
}

#[wasm_bindgen]
impl Tiktoken {
    #[wasm_bindgen(constructor)]
    pub fn new(tiktoken_bfe: &str, special_tokens: JsValue, pat_str: &str) -> Self {
        let constructor = CoreBPEConstructor::new(
            tiktoken_bfe,
            special_tokens.into_serde::<HashMap<String, usize>>().ok(),
            pat_str,
        );

        Tiktoken {
            name: None,
            special_tokens_set: constructor
                .special_tokens
                .keys()
                .map(|s| s.clone())
                .collect(),
            bpe: CoreBPE::new(
                constructor.encoder,
                constructor.special_tokens,
                &constructor.pat_str,
            )
            .unwrap(),
        }
    }

    #[cfg(feature = "inline")]
    fn with_encoding(
        encoding: &str,
        extend_special_tokens: &Option<HashMap<String, usize>>,
    ) -> Result<Self, JsError> {
        let mut constructor: CoreBPEConstructor = match encoding {
            "gpt2" => Ok(CoreBPEConstructor::gpt2()),
            "r50k_base" => Ok(CoreBPEConstructor::r50k_base()),
            "p50k_base" => Ok(CoreBPEConstructor::p50k_base()),
            "p50k_edit" => Ok(CoreBPEConstructor::p50k_edit()),
            "cl100k_base" => Ok(CoreBPEConstructor::cl100k_base()),
            &_ => Err(JsError::new("Invalid encoding")),
        }?;

        if let Some(tokens) = extend_special_tokens {
            constructor.special_tokens.extend(tokens.clone());
        }

        Ok(Tiktoken {
            name: Some(String::from(encoding)),
            // TODO: can we avoid cloning here?
            special_tokens_set: constructor
                .special_tokens
                .keys()
                .map(|s| s.clone())
                .collect(),
            bpe: CoreBPE::new(
                constructor.encoder,
                constructor.special_tokens,
                &constructor.pat_str,
            )
            .unwrap(),
        })
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn encode(
        &self,
        text: &str,
        allowed_special: JsValue,
        disallowed_special: JsValue,
    ) -> Result<Vec<usize>, JsError> {
        let allowed_tokens =
            self.validate_allowed_tokens(text, &allowed_special, &disallowed_special)?;

        Ok(self
            .bpe
            .encode(&text, allowed_tokens.iter().map(AsRef::as_ref).collect()))
    }

    pub fn encode_ordinary(&self, text: &str) -> Vec<usize> {
        self.bpe.encode_ordinary(&text)
    }

    pub fn encode_with_unstable(
        &self,
        text: &str,
        allowed_special: JsValue,
        disallowed_special: JsValue,
    ) -> Result<JsValue, JsError> {
        let allowed_tokens =
            self.validate_allowed_tokens(text, &allowed_special, &disallowed_special)?;

        JsValue::from_serde(
            &self
                .bpe
                .encode_with_unstable(&text, allowed_tokens.iter().map(AsRef::as_ref).collect()),
        )
        .map_err(|e| {
            JsError::new(&format!(
                "Failed to serialize encode_with_unstable result: {}",
                e
            ))
        })
    }

    pub fn encode_single_token(&self, bytes: &[u8]) -> usize {
        self.bpe.encode_single_token(&bytes).unwrap_throw()
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn _encode_single_piece(&self, bytes: &[u8]) -> Vec<usize> {
        self.bpe.encode_single_piece(&bytes)
    }

    pub fn decode(&self, tokens: Vec<usize>) -> Vec<u8> {
        self.bpe.decode_bytes(tokens)
    }

    pub fn decode_single_token_bytes(&self, token: usize) -> Vec<u8> {
        self.bpe.decode_single_token_bytes(token).unwrap_throw()
    }

    pub fn token_byte_values(&self) -> JsValue {
        JsValue::from_serde(&self.bpe.token_byte_values()).unwrap_throw()
    }

    fn validate_allowed_tokens(
        &self,
        text: &str,
        allowed_special_param: &JsValue,
        disallowed_special_param: &JsValue,
    ) -> Result<HashSet<String>, JsError> {
        let allowed_special: HashSet<String> = match allowed_special_param.as_string() {
            Some(value) => match value.as_str() {
                "all" => Ok(self.special_tokens_set.clone()),
                _ => Err(JsError::new("Invalid value for allowed_special")),
            },
            _ => Ok(JsValue::into_serde(&allowed_special_param).unwrap_or_default()),
        }?;

        let disallowed_special = JsValue::into_serde::<HashSet<String>>(&disallowed_special_param)
            .or_else(|_| {
                match disallowed_special_param
                    .as_string()
                    .unwrap_or(String::from("all"))
                    .as_str()
                {
                    "all" => Ok(&self.special_tokens_set - &allowed_special),
                    _ => Err(JsError::new("Invalid value for disallowed_special")),
                }
            })?;

        if !disallowed_special.is_empty() {
            if let Some(found) = Tiktoken::special_token_regex(&disallowed_special).find(text)? {
                return Err(JsError::new(&format!(
                    "The text contains a special token that is not allowed: {}",
                    found.as_str()
                )));
            }
        }

        return Ok(allowed_special);
    }

    fn special_token_regex(tokens: &HashSet<String>) -> Regex {
        let inner = tokens
            .iter()
            .map(|token| regex::escape(token))
            .collect::<Vec<String>>()
            .join("|");

        Regex::new(&format!("({})", inner)).unwrap_throw()
    }
}

#[cfg(feature = "inline")]
#[wasm_bindgen(typescript_custom_section)]
const _: &'static str = r#"
export type TiktokenEmbedding = "gpt2" | "r50k_base" | "p50k_base" | "p50k_edit" | "cl100k_base"; 

/**
 * @param {TiktokenEmbedding} encoding
 * @param {Record<string, number>} [extend_special_tokens]
 * @returns {Tiktoken}
 */
export function get_encoding(encoding: TiktokenEmbedding, extend_special_tokens?: Record<string, number>): Tiktoken;
"#;

#[cfg(feature = "inline")]
#[wasm_bindgen(skip_typescript)]
pub fn get_encoding(encoding: &str, extend_special_tokens: JsValue) -> Result<Tiktoken, JsError> {
    Tiktoken::with_encoding(
        encoding,
        &extend_special_tokens
            .into_serde::<HashMap<String, usize>>()
            .ok(),
    )
}

#[cfg(feature = "inline")]
#[wasm_bindgen(typescript_custom_section)]
const _: &'static str = r#"
export type TiktokenModel =
    | "text-davinci-003"
    | "text-davinci-002"
    | "text-davinci-001"
    | "text-curie-001"
    | "text-babbage-001"
    | "text-ada-001"
    | "davinci"
    | "curie"
    | "babbage"
    | "ada"
    | "code-davinci-002"
    | "code-davinci-001"
    | "code-cushman-002"
    | "code-cushman-001"
    | "davinci-codex"
    | "cushman-codex"
    | "text-davinci-edit-001"
    | "code-davinci-edit-001"
    | "text-embedding-ada-002"
    | "text-similarity-davinci-001"
    | "text-similarity-curie-001"
    | "text-similarity-babbage-001"
    | "text-similarity-ada-001"
    | "text-search-davinci-doc-001"
    | "text-search-curie-doc-001"
    | "text-search-babbage-doc-001"
    | "text-search-ada-doc-001"
    | "code-search-babbage-code-001"
    | "code-search-ada-code-001"
    | "gpt2";

/**
 * @param {TiktokenModel} encoding
 * @param {Record<string, number>} [extend_special_tokens]
 * @returns {Tiktoken}
 */
export function encoding_for_model(model: TiktokenModel, extend_special_tokens?: Record<string, number>): Tiktoken;
"#;

#[cfg(feature = "inline")]
#[wasm_bindgen(skip_typescript)]
pub fn encoding_for_model(
    model: &str,
    extend_special_tokens: JsValue,
) -> Result<Tiktoken, JsError> {
    let encoding = match model {
        "text-davinci-003" => Ok("p50k_base"),
        "text-davinci-002" => Ok("p50k_base"),
        "text-davinci-001" => Ok("r50k_base"),
        "text-curie-001" => Ok("r50k_base"),
        "text-babbage-001" => Ok("r50k_base"),
        "text-ada-001" => Ok("r50k_base"),
        "davinci" => Ok("r50k_base"),
        "curie" => Ok("r50k_base"),
        "babbage" => Ok("r50k_base"),
        "ada" => Ok("r50k_base"),
        "code-davinci-002" => Ok("p50k_base"),
        "code-davinci-001" => Ok("p50k_base"),
        "code-cushman-002" => Ok("p50k_base"),
        "code-cushman-001" => Ok("p50k_base"),
        "davinci-codex" => Ok("p50k_base"),
        "cushman-codex" => Ok("p50k_base"),
        "text-davinci-edit-001" => Ok("p50k_edit"),
        "code-davinci-edit-001" => Ok("p50k_edit"),
        "text-embedding-ada-002" => Ok("cl100k_base"),
        "text-similarity-davinci-001" => Ok("r50k_base"),
        "text-similarity-curie-001" => Ok("r50k_base"),
        "text-similarity-babbage-001" => Ok("r50k_base"),
        "text-similarity-ada-001" => Ok("r50k_base"),
        "text-search-davinci-doc-001" => Ok("r50k_base"),
        "text-search-curie-doc-001" => Ok("r50k_base"),
        "text-search-babbage-doc-001" => Ok("r50k_base"),
        "text-search-ada-doc-001" => Ok("r50k_base"),
        "code-search-babbage-code-001" => Ok("r50k_base"),
        "code-search-ada-code-001" => Ok("r50k_base"),
        "gpt2" => Ok("gpt2"),
        model => Err(JsError::new(
            format!("Invalid model: {}", model.to_string()).as_str(),
        )),
    }?;

    Tiktoken::with_encoding(
        encoding,
        &extend_special_tokens
            .into_serde::<HashMap<String, usize>>()
            .ok(),
    )
}

fn _byte_pair_merge(piece: &[u8], ranks: &HashMap<Vec<u8>, usize>) -> Vec<std::ops::Range<usize>> {
    let mut parts: Vec<_> = (0..piece.len()).map(|i| i..i + 1).collect();

    // If you have n parts and m merges, this does O(mn) work
    // We could do something with a heap and do O(m log n) work

    // Note that we hash bytes, not token pairs. As long as we train BPE the way we
    // currently do, this is equivalent. An easy way to break this would be to decouple
    // merge priority from token index or to prevent specific token merges.
    loop {
        if parts.len() == 1 {
            break;
        }
        let mut min_rank: Option<(usize, usize)> = None;
        for i in 0..parts.len() - 1 {
            let rank = if let Some(r) = ranks.get(&piece[parts[i].start..parts[i + 1].end]) {
                *r
            } else {
                continue;
            };
            if min_rank.is_none() || rank < min_rank.unwrap().0 {
                min_rank = Some((rank, i));
            }
        }
        if let Some((_, i)) = min_rank {
            parts[i] = parts[i].start..parts[i + 1].end;
            parts.remove(i + 1);
        } else {
            break;
        }
    }
    parts
}

pub fn byte_pair_encode(piece: &[u8], ranks: &HashMap<Vec<u8>, usize>) -> Vec<usize> {
    if piece.len() == 1 {
        return vec![ranks[piece]];
    }
    _byte_pair_merge(piece, ranks)
        .iter()
        .map(|p| ranks[&piece[p.start..p.end]])
        .collect()
}

pub fn byte_pair_split<'a>(piece: &'a [u8], ranks: &HashMap<Vec<u8>, usize>) -> Vec<&'a [u8]> {
    if piece.len() == 1 {
        return vec![piece];
    }
    _byte_pair_merge(piece, ranks)
        .iter()
        .map(|p| &piece[p.start..p.end])
        .collect()
}

// Various performance notes:
//
// Regex
// =====
// Most of the time is spent in regex. The easiest way to speed this up is by using less fancy
// regex features. For instance, using a regex parse-able by `regex` crate is 3x faster than
// the usual regex we use.
//
// However, given that we're using a regex parse-able by `regex`, there isn't much difference
// between using the `regex` crate and using the `fancy_regex` crate.
//
// There is an important interaction between threading, `regex` and `fancy_regex`.
// When using `fancy_regex`, we hit `regex.find_at`. It turns out that this causes contention on
// some mutable scratch space inside of `regex`. This absolutely kills performance. When using plain
// old `regex`, we don't hit this, because `find_iter` has a different code path.
// Related: https://github.com/rust-lang/regex/blob/master/PERFORMANCE.md
// Anyway, the way we get around this is with having a (mostly) thread local clone of the regex for
// each thread.
//
// Threading
// =========
// I tried using `rayon`. It wasn't really faster than using Python threads and releasing the GIL.
// So goodbye `rayon`! Let thread count etc be in control of our Python users.
//
// Caching
// =======
// The reference tokeniser has an lru cache over the equivalent of `byte_pair_encode`.
// Originally, we had one too! Without it, we were only vaguely faster than Python.
// I used an RWLock to protect the cache. This didn't seem to hurt single threaded performance
// noticeably, but it did affect multi-threaded performance. Weirdly, it seemed to affect
// multi-threaded performance even when I only had readers (maybed I messed something up?).
// Anyway, I realised that we could get rid of the cache, if we treat the set of tokens as a cache!
// These are exactly the set or merges that are likely to be hot. And now we don't have to think
// about interior mutability, memory use, or cloning.
//
// Hashing
// =======
// We use FxHashMap instead of the standard HashMap. This is maybe like a 5-10% win?
// The current implementation ends up doing a lot of hashing of bytes. In theory, this could be made
// to be hashing of two-tuples of ints, which looks like it may also be a couple percent faster.

use std::num::NonZeroU64;
pub struct FakeThreadId(NonZeroU64);

struct CoreBPE {
    encoder: HashMap<Vec<u8>, usize>,
    special_tokens_encoder: HashMap<String, usize>,
    decoder: HashMap<usize, Vec<u8>>,
    special_tokens_decoder: HashMap<usize, Vec<u8>>,
    regex: Regex,
    special_regex: Regex,
    sorted_token_bytes: Vec<Vec<u8>>,
}

impl CoreBPE {
    fn _get_tl_regex(&self) -> &Regex {
        // See performance notes above for what this is about
        // It's also a little janky, please make a better version of it!
        // However, it's nice that this doesn't leak memory to short-lived threads
        &self.regex
    }

    fn _get_tl_special_regex(&self) -> &Regex {
        &self.special_regex
    }

    fn _decode_native(&self, tokens: &[usize]) -> Vec<u8> {
        let mut ret = Vec::with_capacity(tokens.len() * 2);
        for token in tokens {
            let token_bytes = self
                .decoder
                .get(token)
                .unwrap_or_else(|| &self.special_tokens_decoder[token]);
            ret.extend(token_bytes);
        }
        ret
    }

    fn _encode_ordinary_native(&self, text: &str) -> Vec<usize> {
        // This is the core of the encoding logic; the other functions in here
        // just make things complicated :-)
        let regex = self._get_tl_regex();
        let mut ret = vec![];
        for mat in regex.find_iter(text) {
            let piece = mat.unwrap().as_str().as_bytes();
            if let Some(token) = self.encoder.get(piece) {
                ret.push(*token);
                continue;
            }
            ret.extend(&byte_pair_encode(piece, &self.encoder));
        }
        ret
    }

    fn _encode_native(&self, text: &str, allowed_special: &HashSet<&str>) -> (Vec<usize>, usize) {
        let special_regex = self._get_tl_special_regex();
        let regex = self._get_tl_regex();
        let mut ret = vec![];

        let mut start = 0;
        let mut last_piece_token_len = 0;
        loop {
            let mut next_special;
            let mut start_find = start;
            loop {
                // Find the next allowed special token, if any
                next_special = special_regex.find_from_pos(text, start_find).unwrap();
                match next_special {
                    Some(m) => {
                        if allowed_special.contains(&text[m.start()..m.end()]) {
                            break;
                        }
                        start_find = m.start() + 1;
                    }
                    None => break,
                }
            }
            let end = next_special.map_or(text.len(), |m| m.start());

            // Okay, here we go, compare this logic to _encode_ordinary_native
            for mat in regex.find_iter(&text[start..end]) {
                let piece = mat.unwrap().as_str().as_bytes();
                if let Some(token) = self.encoder.get(piece) {
                    last_piece_token_len = 1;
                    ret.push(*token);
                    continue;
                }
                let tokens = byte_pair_encode(piece, &self.encoder);
                last_piece_token_len = tokens.len();
                ret.extend(&tokens);
            }

            match next_special {
                // And here we push the special token
                Some(m) => {
                    let piece = m.as_str();
                    let token = self.special_tokens_encoder[piece];
                    ret.push(token);
                    start = m.end();
                    last_piece_token_len = 0;
                }
                None => break,
            }
        }

        // last_piece_token_len is how many tokens came from the last regex split. This is used
        // for determining unstable tokens, since you can't merge across (stable) regex splits
        (ret, last_piece_token_len)
    }

    fn _increase_last_piece_token_len(
        &self,
        tokens: Vec<usize>,
        mut last_piece_token_len: usize,
    ) -> (Vec<usize>, usize) {
        // Unfortunately, the locations where our regex splits can be unstable.
        // For the purposes of determining unstable tokens, unstable regex splitting
        // is only a problem if a split that was present disappears, since this can
        // lead to merging of tokens otherwise thought to be stable.
        // cl100k_base makes our life hard by including the \s*[\r\n]+
        // pattern. This can e.g. cause "\n" + " " to become "\n \n".
        // Here is a quick and dirty fix:
        {
            let token_is_all_space = |token| {
                self.decoder
                    .get(token)
                    .map(|token_bytes| {
                        token_bytes
                            .iter()
                            .rev()
                            .all(|&b| [b' ', b'\n', b'\t'].contains(&b))
                    })
                    .unwrap_or(false)
            };
            if last_piece_token_len > 0
                && token_is_all_space(&tokens[tokens.len() - last_piece_token_len])
            {
                while (last_piece_token_len < tokens.len())
                    && token_is_all_space(&tokens[tokens.len() - last_piece_token_len - 1])
                {
                    last_piece_token_len += 1;
                }
            }
        }
        debug_assert!(last_piece_token_len <= tokens.len());

        (tokens, last_piece_token_len)
    }

    fn _encode_unstable_native(
        &self,
        text: &str,
        allowed_special: &HashSet<&str>,
    ) -> (Vec<usize>, HashSet<Vec<usize>>) {
        let (tokens, last_piece_token_len) = self._encode_native(text, allowed_special);
        if last_piece_token_len == 0 {
            // If last_piece_token_len is zero, the last token was a special token and we have
            // no unstable bytes
            return (tokens, HashSet::new());
        }
        let (mut tokens, last_piece_token_len) =
            self._increase_last_piece_token_len(tokens, last_piece_token_len);

        let unstable_bytes = self._decode_native(&tokens[tokens.len() - last_piece_token_len..]);
        tokens.truncate(tokens.len() - last_piece_token_len);

        // TODO: we should try harder to find additional stable tokens
        // This would reduce the amount of retokenising when determining completions
        // Refer to the logic in an older version of this file

        let mut completions = HashSet::new();
        if unstable_bytes.is_empty() {
            return (tokens, completions);
        }

        // This is the easy bit. Just find all single tokens that start with unstable_bytes
        // (including tokens that exactly match unstable_bytes)
        // Separating this from the loop below helps with performance in a common case.
        let mut point = self
            .sorted_token_bytes
            .partition_point(|x| x.as_slice() < unstable_bytes.as_slice());
        while point < self.sorted_token_bytes.len()
            && self.sorted_token_bytes[point].starts_with(&unstable_bytes)
        {
            completions.insert(vec![
                self.encoder[self.sorted_token_bytes[point].as_slice()],
            ]);
            point += 1;
        }

        // Now apply even more brute force. At every (other) possible position for the straddling
        // token, concatenate additional bytes from that token (if any) to unstable_bytes,
        // and retokenise the whole thing and see what we get.
        for i in 1..unstable_bytes.len() {
            let prefix = &unstable_bytes[..i];
            let suffix = &unstable_bytes[i..];
            let mut point = self
                .sorted_token_bytes
                .partition_point(|x| x.as_slice() < suffix);
            // TODO: Perf optimisation if suffix starts with " "?
            while point < self.sorted_token_bytes.len()
                && self.sorted_token_bytes[point].starts_with(suffix)
            {
                let possibility = [prefix, self.sorted_token_bytes[point].as_slice()].concat();
                let encoded = match std::str::from_utf8(&possibility) {
                    // Morally, this is byte_pair_encode(&possibility, &self.encoder)
                    // But we might have introduced a regex split which would prevent merges.
                    // (particularly possible in the presence of unstable regex splits)
                    // So convert to UTF-8 and do regex splitting.
                    // E.g. with cl100k_base "  !" gets split to " " + " !",
                    // but byte_pair_encode("  !") != byte_pair_encode(" ")
                    Ok(s) => self._encode_ordinary_native(s),

                    // Technically, whether or not this arm is correct depends on whether there
                    // would be a regex split before the UTF-8 truncation point.
                    // Probably niche enough that no one will ever notice (after all, people didn't
                    // notice all the big holes in the previous unstable token implementation)
                    Err(_) => byte_pair_encode(&possibility, &self.encoder),
                    // Something like the following is intriguing but incorrect:
                    // Err(e) => self._encode_ordinary_native(unsafe {
                    //     std::str::from_utf8_unchecked(&possibility[..e.valid_up_to()])
                    // }),
                };
                let mut seq = Vec::new();
                let mut seq_len = 0;
                for token in encoded {
                    seq.push(token);
                    seq_len += self.decoder[&token].len();
                    if seq_len >= unstable_bytes.len() {
                        break;
                    }
                }
                completions.insert(seq);
                point += 1;
            }
        }

        // This is also not straightforward. While we generally assume that regex splits are stable,
        // unfortunately, they are not. That is, if adding bytes were to make a split appear in
        // unstable_bytes, this could make tokens possible which our logic would otherwise think
        // would be merged.
        // For example, with gpt2, the use of \s+(?!\S) means that "\n\n" could
        // develop a split, e.g. "\n\n0" splits into "\n"+"\n"+"0", making "\n" a possible token.
        // Here is a quick and dirty fix:
        // This isn't right if we ever remove \s+(?!\S)
        if unstable_bytes.len() > 1 {
            let last_decoded = bstr::decode_last_utf8(unstable_bytes.as_slice());
            if unstable_bytes.len() - last_decoded.1 > 0
                && last_decoded.0.map_or(false, |c| c.is_whitespace())
            {
                let mut reencoded = byte_pair_encode(
                    &unstable_bytes[..unstable_bytes.len() - last_decoded.1],
                    &self.encoder,
                );
                reencoded.extend(byte_pair_encode(
                    &unstable_bytes[unstable_bytes.len() - last_decoded.1..],
                    &self.encoder,
                ));
                completions.insert(reencoded);
            }
        }

        (tokens, completions)
    }
}

impl CoreBPE {
    fn new(
        encoder: HashMap<Vec<u8>, usize>,
        special_tokens_encoder: HashMap<String, usize>,
        pattern: &str,
    ) -> Result<Self, Error> {
        let regex = Regex::new(pattern)?;

        let special_regex = {
            let _parts = special_tokens_encoder
                .keys()
                .map(|s| fancy_regex::escape(s))
                .collect::<Vec<_>>();
            Regex::new(&_parts.join("|"))?
        };

        let decoder: HashMap<usize, Vec<u8>> =
            encoder.iter().map(|(k, v)| (*v, k.clone())).collect();

        assert!(encoder.len() == decoder.len());

        let special_tokens_decoder: HashMap<usize, Vec<u8>> = special_tokens_encoder
            .iter()
            .map(|(k, v)| (*v, k.as_bytes().to_vec()))
            .collect();

        // Clone because I don't know how to tell Rust I'm not going to change the map
        let mut sorted_token_bytes: Vec<Vec<u8>> = encoder.keys().cloned().collect();
        sorted_token_bytes.sort();

        Ok(CoreBPE {
            encoder,
            special_tokens_encoder,
            decoder,
            special_tokens_decoder,
            regex,
            special_regex,
            sorted_token_bytes,
        })
    }

    // ====================
    // Encoding
    // ====================

    fn encode_ordinary(&self, text: &str) -> Vec<usize> {
        self._encode_ordinary_native(text)
    }

    fn encode(&self, text: &str, allowed_special: HashSet<&str>) -> Vec<usize> {
        self._encode_native(text, &allowed_special).0
    }

    fn _encode_bytes(&self, bytes: &[u8]) -> Vec<usize> {
        {
            match std::str::from_utf8(bytes) {
                Ok(text) => self._encode_ordinary_native(text),
                Err(e) => {
                    let text = unsafe { std::str::from_utf8_unchecked(&bytes[..e.valid_up_to()]) };
                    let (tokens, last_piece_token_len) = self._encode_native(text, &HashSet::new());
                    let (mut tokens, last_piece_token_len) =
                        self._increase_last_piece_token_len(tokens, last_piece_token_len);
                    if !tokens.is_empty() && last_piece_token_len > 0 {
                        // Lop off the tokens from the last piece and run BPE on the remaining bytes
                        // Somewhat niche, but this may not be correct if we'd have had a regex
                        // split between the valid UTF-8 and the invalid bytes, which is why this
                        // method is private
                        let mut unstable_bytes =
                            self._decode_native(&tokens[tokens.len() - last_piece_token_len..]);
                        unstable_bytes.extend_from_slice(&bytes[e.valid_up_to()..]);

                        tokens.truncate(tokens.len() - last_piece_token_len);
                        tokens.extend(byte_pair_encode(&unstable_bytes, &self.encoder));
                    }
                    tokens
                }
            }
        }
    }

    fn encode_with_unstable(
        &self,
        text: &str,
        allowed_special: HashSet<&str>,
    ) -> (Vec<usize>, HashSet<Vec<usize>>) {
        self._encode_unstable_native(text, &allowed_special)
    }

    fn encode_single_token(&self, piece: &[u8]) -> Result<usize, Error> {
        if let Some(token) = self.encoder.get(piece).copied() {
            return Ok(token);
        }
        if let Ok(piece_str) = std::str::from_utf8(piece) {
            if let Some(token) = self.special_tokens_encoder.get(piece_str).copied() {
                return Ok(token);
            }
        }
        Err(anyhow!("Unable to encode single token: {:?}", piece))
    }

    fn encode_single_piece(&self, piece: &[u8]) -> Vec<usize> {
        if let Some(token) = self.encoder.get(piece) {
            return vec![*token];
        }
        byte_pair_encode(piece, &self.encoder)
    }

    // ====================
    // Decoding
    // ====================

    fn decode_bytes(&self, tokens: Vec<usize>) -> Vec<u8> {
        self._decode_native(&tokens)
    }

    fn decode_single_token_bytes(&self, token: usize) -> Result<Vec<u8>, Error> {
        if let Some(bytes) = self.decoder.get(&token) {
            return Ok(bytes.clone());
        }
        if let Some(bytes) = self.special_tokens_decoder.get(&token) {
            return Ok(bytes.clone());
        }
        Err(anyhow!(
            "Token not found in the vocabulary: {}",
            token.to_string()
        ))
    }

    // ====================
    // Miscellaneous
    // ====================

    fn token_byte_values(&self) -> Vec<Vec<u8>> {
        self.sorted_token_bytes.clone()
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap as HashMap;

    use crate::byte_pair_split;

    #[test]
    fn very_simple_test() {
        let mut ranks = HashMap::default();
        ranks.insert(b"ab".to_vec(), 1);
        ranks.insert(b"cd".to_vec(), 2);

        let res = byte_pair_split(b"abcd", &ranks);
        assert_eq!(res, vec![b"ab", b"cd"]);
    }
}
