use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

/// 文本预处理器
/// 负责文本规范化、音素转换和音素 ID 映射
#[derive(Clone)]
pub struct TextProcessor {
    /// 音素到 ID 的映射
    phone_to_id: HashMap<String, i64>,
    /// ID 到音素的映射（反向）
    id_to_phone: HashMap<i64, String>,
    /// 默认语言
    locale: String,
}

impl TextProcessor {
    /// 从模型目录加载文本预处理器
    pub fn new_from_dir(model_dir: &Path, locale: &str) -> Result<Self> {
        let phone_map_path = model_dir.join("fastspeech2-lite").join("phone_id_map.txt");
        
        if !phone_map_path.exists() {
            return Err(anyhow!("phone_id_map.txt not found at {}", phone_map_path.display()));
        }

        // 读取 phone_id_map.txt
        let content = fs::read_to_string(&phone_map_path)
            .map_err(|e| anyhow!("failed to read phone_id_map.txt: {e}"))?;

        let mut phone_to_id = HashMap::new();
        let mut id_to_phone = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let phone = parts[0].to_string();
                let id: i64 = parts[1].parse()
                    .map_err(|e| anyhow!("invalid phone ID in phone_id_map.txt: {}: {e}", parts[1]))?;
                
                phone_to_id.insert(phone.clone(), id);
                id_to_phone.insert(id, phone);
            }
        }

        Ok(Self {
            phone_to_id,
            id_to_phone,
            locale: locale.to_string(),
        })
    }

    /// 规范化文本（简化版：目前只做基本处理）
    pub fn normalize_text(&self, text: &str) -> String {
        let mut result = text.trim().to_string();
        
        if result.is_empty() {
            return result;
        }
        
        // 移除多余空格
        result = result.split_whitespace().collect::<Vec<&str>>().join(" ");
        
        // 基本标点符号处理（保留常见标点）
        // 移除特殊字符（保留字母、数字、常见标点）
        result = result.chars()
            .filter(|c| c.is_alphanumeric() || 
                       matches!(c, ' ' | '.' | ',' | '!' | '?' | ';' | ':' | '-' | '\'' | '"'))
            .collect();
        
        // TODO: 实现更复杂的规范化
        // - 数字转文字（"123" → "一百二十三" 或 "one hundred twenty three"）
        // - 日期时间处理
        // - 缩写展开（"Dr." → "Doctor"）
        // - 货币、单位转换
        
        result
    }

    /// 将文本转换为音素序列
    /// 
    /// 实现：
    /// - 中文：文本 → 拼音 → 音素
    /// - 英文：文本 → 音素（使用常见单词映射 + 基于规则的转换）
    pub fn text_to_phonemes(&self, text: &str) -> Result<Vec<String>> {
        let normalized = self.normalize_text(text);
        
        if normalized.is_empty() {
            return Ok(vec![]);
        }
        
        match self.locale.as_str() {
            "zh" | "chinese" | "zh-CN" => {
                self.text_to_phonemes_chinese(&normalized)
            }
            "en" | "english" | "en-US" => {
                self.text_to_phonemes_english(&normalized)
            }
            _ => Err(anyhow!("Unsupported locale for text-to-phoneme: {}", self.locale)),
        }
    }

    /// 中文文本转音素：文本 → 拼音 → 音素
    fn text_to_phonemes_chinese(&self, text: &str) -> Result<Vec<String>> {
        let mut phonemes = Vec::new();
        
        for ch in text.chars() {
            if ch.is_whitespace() {
                continue; // 跳过空格
            }
            
            // 尝试将中文字符转换为拼音
            if let Some(pinyin) = self.chinese_char_to_pinyin(ch) {
                // 将拼音转换为音素序列
                let pinyin_phonemes = self.pinyin_to_phonemes(&pinyin);
                phonemes.extend(pinyin_phonemes);
            } else {
                // 如果无法转换，使用 <unk>
                println!("[DEBUG TextProcessor] Character '{}' cannot be converted to pinyin, using <unk>", ch);
                phonemes.push("<unk>".to_string());
            }
        }
        
        Ok(phonemes)
    }

    /// 英文文本转音素：使用常见单词映射 + 基于规则的转换
    fn text_to_phonemes_english(&self, text: &str) -> Result<Vec<String>> {
        let mut phonemes = Vec::new();
        
        // 按单词分割
        for word in text.split_whitespace() {
            let word_lower = word.to_lowercase();
            
            // 1. 尝试查找常见单词的音素映射
            if let Some(word_phonemes) = self.get_common_word_phonemes(&word_lower) {
                phonemes.extend(word_phonemes);
                continue;
            }
            
            // 2. 基于规则的音素转换（简化版）
            let rule_based_phonemes = self.word_to_phonemes_by_rules(&word_lower);
            if !rule_based_phonemes.is_empty() {
                phonemes.extend(rule_based_phonemes);
            } else {
                // 3. 如果都失败，使用 <unk>
                println!("[DEBUG TextProcessor] Word '{}' cannot be converted to phonemes, using <unk>", word_lower);
                phonemes.push("<unk>".to_string());
            }
        }
        
        Ok(phonemes)
    }

    /// 常见英文单词的音素映射（简化版词典）
    fn get_common_word_phonemes(&self, word: &str) -> Option<Vec<String>> {
        // 常见单词的音素映射（CMUdict 风格）
        // 使用静态数组来避免类型问题
        static COMMON_WORDS: &[(&str, &[&str])] = &[
            ("hello", &["HH", "EH", "L", "OW"]),
            ("hi", &["HH", "AY"]),
            ("how", &["HH", "AW"]),
            ("are", &["AA", "R"]),
            ("you", &["Y", "UW"]),
            ("yes", &["Y", "EH", "S"]),
            ("no", &["N", "OW"]),
            ("the", &["DH", "AH"]),
            ("a", &["AH"]),
            ("an", &["AE", "N"]),
            ("is", &["IH", "Z"]),
            ("this", &["DH", "IH", "S"]),
            ("that", &["DH", "AE", "T"]),
            ("what", &["W", "AH", "T"]),
            ("where", &["W", "EH", "R"]),
            ("when", &["W", "EH", "N"]),
            ("who", &["HH", "UW"]),
            ("why", &["W", "AY"]),
            ("good", &["G", "UH", "D"]),
            ("bad", &["B", "AE", "D"]),
            ("ok", &["OW", "K", "EY"]),
            ("okay", &["OW", "K", "EY"]),
            ("thanks", &["TH", "AE", "NG", "K", "S"]),
            ("thank", &["TH", "AE", "NG", "K"]),
            ("please", &["P", "L", "IY", "Z"]),
            ("sorry", &["S", "AA", "R", "IY"]),
            ("excuse", &["IH", "K", "S", "K", "Y", "UW", "Z"]),
            ("me", &["M", "IY"]),
            ("my", &["M", "AY"]),
            ("name", &["N", "EY", "M"]),
            ("i", &["AY"]),
            ("am", &["AE", "M"]),
            ("to", &["T", "UW"]),
            ("go", &["G", "OW"]),
            ("come", &["K", "AH", "M"]),
            ("see", &["S", "IY"]),
            ("say", &["S", "EY"]),
            ("speak", &["S", "P", "IY", "K"]),
            ("talk", &["T", "AO", "K"]),
            ("listen", &["L", "IH", "S", "AH", "N"]),
            ("hear", &["HH", "IH", "R"]),
            ("understand", &["AH", "N", "D", "ER", "S", "T", "AE", "N", "D"]),
            ("know", &["N", "OW"]),
            ("think", &["TH", "IH", "NG", "K"]),
            ("want", &["W", "AA", "N", "T"]),
            ("need", &["N", "IY", "D"]),
            ("like", &["L", "AY", "K"]),
            ("love", &["L", "AH", "V"]),
            ("hate", &["HH", "EY", "T"]),
            ("help", &["HH", "EH", "L", "P"]),
            ("can", &["K", "AE", "N"]),
            ("could", &["K", "UH", "D"]),
            ("would", &["W", "UH", "D"]),
            ("should", &["SH", "UH", "D"]),
            ("will", &["W", "IH", "L"]),
            ("do", &["D", "UW"]),
            ("does", &["D", "AH", "Z"]),
            ("did", &["D", "IH", "D"]),
            ("have", &["HH", "AE", "V"]),
            ("has", &["HH", "AE", "Z"]),
            ("had", &["HH", "AE", "D"]),
            ("get", &["G", "EH", "T"]),
            ("got", &["G", "AA", "T"]),
            ("give", &["G", "IH", "V"]),
            ("take", &["T", "EY", "K"]),
            ("make", &["M", "EY", "K"]),
            ("let", &["L", "EH", "T"]),
            ("put", &["P", "UH", "T"]),
            ("find", &["F", "AY", "N", "D"]),
            ("look", &["L", "UH", "K"]),
            ("watch", &["W", "AA", "CH"]),
            ("read", &["R", "IY", "D"]),
            ("write", &["R", "AY", "T"]),
            ("eat", &["IY", "T"]),
            ("drink", &["D", "R", "IH", "NG", "K"]),
            ("sleep", &["S", "L", "IY", "P"]),
            ("wake", &["W", "EY", "K"]),
            ("work", &["W", "ER", "K"]),
            ("play", &["P", "L", "EY"]),
            ("run", &["R", "AH", "N"]),
            ("walk", &["W", "AO", "K"]),
            ("sit", &["S", "IH", "T"]),
            ("stand", &["S", "T", "AE", "N", "D"]),
            ("open", &["OW", "P", "AH", "N"]),
            ("close", &["K", "L", "OW", "Z"]),
            ("start", &["S", "T", "AA", "R", "T"]),
            ("stop", &["S", "T", "AA", "P"]),
            ("begin", &["B", "IH", "G", "IH", "N"]),
            ("end", &["EH", "N", "D"]),
            ("finish", &["F", "IH", "N", "IH", "SH"]),
            ("continue", &["K", "AH", "N", "T", "IH", "N", "Y", "UW"]),
            ("try", &["T", "R", "AY"]),
            ("use", &["Y", "UW", "Z"]),
            ("call", &["K", "AO", "L"]),
            ("ask", &["AE", "S", "K"]),
            ("answer", &["AE", "N", "S", "ER"]),
            ("tell", &["T", "EH", "L"]),
            ("show", &["SH", "OW"]),
            ("learn", &["L", "ER", "N"]),
            ("teach", &["T", "IY", "CH"]),
            ("study", &["S", "T", "AH", "D", "IY"]),
            ("remember", &["R", "IH", "M", "EH", "M", "B", "ER"]),
            ("forget", &["F", "ER", "G", "EH", "T"]),
            ("meet", &["M", "IY", "T"]),
            ("leave", &["L", "IY", "V"]),
            ("stay", &["S", "T", "EY"]),
            ("wait", &["W", "EY", "T"]),
            ("hurry", &["HH", "ER", "IY"]),
            ("slow", &["S", "L", "OW"]),
            ("fast", &["F", "AE", "S", "T"]),
            ("quick", &["K", "W", "IH", "K"]),
            ("big", &["B", "IH", "G"]),
            ("small", &["S", "M", "AO", "L"]),
            ("large", &["L", "AA", "R", "JH"]),
            ("little", &["L", "IH", "T", "AH", "L"]),
            ("long", &["L", "AO", "NG"]),
            ("short", &["SH", "AO", "R", "T"]),
            ("tall", &["T", "AO", "L"]),
            ("high", &["HH", "AY"]),
            ("low", &["L", "OW"]),
            ("wide", &["W", "AY", "D"]),
            ("narrow", &["N", "AE", "R", "OW"]),
            ("thick", &["TH", "IH", "K"]),
            ("thin", &["TH", "IH", "N"]),
            ("heavy", &["HH", "EH", "V", "IY"]),
            ("light", &["L", "AY", "T"]),
            ("new", &["N", "UW"]),
            ("old", &["OW", "L", "D"]),
            ("young", &["Y", "AH", "NG"]),
        ];
        
        COMMON_WORDS.iter()
            .find(|(w, _)| *w == word)
            .map(|(_, phones)| phones.iter().map(|s| s.to_string()).collect())
    }

    /// 基于规则的音素转换（简化版，用于未知单词）
    fn word_to_phonemes_by_rules(&self, word: &str) -> Vec<String> {
        // 这是一个非常简化的实现，只处理一些基本规则
        // 实际应该使用更复杂的音素转换算法或 CMUdict
        
        let mut phonemes = Vec::new();
        let chars: Vec<char> = word.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let ch = chars[i].to_uppercase().next().unwrap_or(chars[i]);
            
            // 处理常见的字母组合
            if i + 1 < chars.len() {
                let next_ch = chars[i + 1].to_uppercase().next().unwrap_or(chars[i + 1]);
                let digraph = format!("{}{}", ch, next_ch);
                
                // 检查是否是已知的音素
                if self.phone_to_id.contains_key(&digraph) {
                    phonemes.push(digraph);
                    i += 2;
                    continue;
                }
            }
            
            // 单个字母
            let single = ch.to_string();
            if self.phone_to_id.contains_key(&single) {
                phonemes.push(single);
            }
            
            i += 1;
        }
        
        phonemes
    }

    /// 中文字符转拼音（简化版）
    fn chinese_char_to_pinyin(&self, ch: char) -> Option<String> {
        // 这是一个非常简化的实现
        // 实际应该使用完整的拼音转换库（如 pypinyin）
        
        // 常见中文字符的拼音映射（简化版，只包含少量字符）
        let char_to_pinyin: std::collections::HashMap<char, &str> = [
            ('你', "ni"),
            ('好', "hao"),
            ('我', "wo"),
            ('是', "shi"),
            ('的', "de"),
            ('了', "le"),
            ('在', "zai"),
            ('有', "you"),
            ('和', "he"),
            ('就', "jiu"),
            ('不', "bu"),
            ('人', "ren"),
            ('这', "zhe"),
            ('中', "zhong"),
            ('大', "da"),
            ('为', "wei"),
            ('上', "shang"),
            ('个', "ge"),
            ('国', "guo"),
            ('我', "wo"),
            ('以', "yi"),
            ('要', "yao"),
            ('他', "ta"),
            ('时', "shi"),
            ('来', "lai"),
            ('用', "yong"),
            ('们', "men"),
            ('生', "sheng"),
            ('到', "dao"),
            ('作', "zuo"),
            ('地', "di"),
            ('于', "yu"),
            ('出', "chu"),
            ('就', "jiu"),
            ('分', "fen"),
            ('对', "dui"),
            ('成', "cheng"),
            ('会', "hui"),
            ('可', "ke"),
            ('主', "zhu"),
            ('发', "fa"),
            ('年', "nian"),
            ('动', "dong"),
            ('同', "tong"),
            ('工', "gong"),
            ('也', "ye"),
            ('能', "neng"),
            ('下', "xia"),
            ('过', "guo"),
            ('子', "zi"),
            ('说', "shuo"),
            ('产', "chan"),
            ('种', "zhong"),
            ('面', "mian"),
            ('而', "er"),
            ('方', "fang"),
            ('后', "hou"),
            ('多', "duo"),
            ('定', "ding"),
            ('行', "xing"),
            ('学', "xue"),
            ('法', "fa"),
            ('所', "suo"),
            ('民', "min"),
            ('得', "de"),
            ('经', "jing"),
            ('十', "shi"),
            ('三', "san"),
            ('之', "zhi"),
            ('进', "jin"),
            ('着', "zhe"),
            ('等', "deng"),
            ('部', "bu"),
            ('度', "du"),
            ('家', "jia"),
            ('电', "dian"),
            ('力', "li"),
            ('里', "li"),
            ('如', "ru"),
            ('水', "shui"),
            ('化', "hua"),
            ('高', "gao"),
            ('自', "zi"),
            ('二', "er"),
            ('理', "li"),
            ('小', "xiao"),
            ('物', "wu"),
            ('现', "xian"),
            ('实', "shi"),
            ('加', "jia"),
            ('量', "liang"),
            ('都', "dou"),
            ('两', "liang"),
            ('体', "ti"),
            ('制', "zhi"),
            ('机', "ji"),
            ('当', "dang"),
            ('使', "shi"),
            ('点', "dian"),
            ('从', "cong"),
            ('业', "ye"),
            ('本', "ben"),
            ('去', "qu"),
            ('把', "ba"),
            ('性', "xing"),
            ('好', "hao"),
            ('应', "ying"),
            ('开', "kai"),
            ('它', "ta"),
            ('合', "he"),
            ('还', "hai"),
            ('因', "yin"),
            ('由', "you"),
            ('其', "qi"),
            ('些', "xie"),
            ('然', "ran"),
            ('前', "qian"),
            ('外', "wai"),
            ('天', "tian"),
            ('政', "zheng"),
            ('四', "si"),
            ('日', "ri"),
            ('那', "na"),
            ('社', "she"),
            ('义', "yi"),
            ('事', "shi"),
            ('平', "ping"),
            ('形', "xing"),
            ('相', "xiang"),
            ('全', "quan"),
            ('表', "biao"),
            ('间', "jian"),
            ('样', "yang"),
            ('与', "yu"),
            ('关', "guan"),
            ('各', "ge"),
            ('重', "zhong"),
            ('新', "xin"),
            ('线', "xian"),
            ('内', "nei"),
            ('数', "shu"),
            ('正', "zheng"),
            ('心', "xin"),
            ('反', "fan"),
            ('你', "ni"),
            ('明', "ming"),
            ('看', "kan"),
            ('原', "yuan"),
            ('又', "you"),
            ('么', "me"),
            ('利', "li"),
            ('比', "bi"),
            ('或', "huo"),
            ('但', "dan"),
            ('质', "zhi"),
            ('气', "qi"),
            ('第', "di"),
            ('向', "xiang"),
            ('道', "dao"),
            ('命', "ming"),
            ('此', "ci"),
            ('变', "bian"),
            ('条', "tiao"),
            ('只', "zhi"),
            ('没', "mei"),
            ('结', "jie"),
            ('解', "jie"),
            ('问', "wen"),
            ('意', "yi"),
            ('建', "jian"),
            ('月', "yue"),
            ('公', "gong"),
            ('无', "wu"),
            ('系', "xi"),
            ('军', "jun"),
            ('很', "hen"),
            ('情', "qing"),
            ('者', "zhe"),
            ('最', "zui"),
            ('立', "li"),
            ('代', "dai"),
            ('想', "xiang"),
            ('已', "yi"),
            ('通', "tong"),
            ('并', "bing"),
            ('提', "ti"),
            ('直', "zhi"),
            ('题', "ti"),
            ('党', "dang"),
            ('程', "cheng"),
            ('展', "zhan"),
            ('五', "wu"),
            ('果', "guo"),
            ('料', "liao"),
            ('象', "xiang"),
            ('员', "yuan"),
            ('革', "ge"),
            ('位', "wei"),
            ('入', "ru"),
            ('常', "chang"),
            ('文', "wen"),
            ('总', "zong"),
            ('次', "ci"),
            ('品', "pin"),
            ('式', "shi"),
            ('活', "huo"),
            ('设', "she"),
            ('及', "ji"),
            ('管', "guan"),
            ('特', "te"),
            ('件', "jian"),
            ('长', "chang"),
            ('求', "qiu"),
            ('老', "lao"),
            ('头', "tou"),
            ('基', "ji"),
            ('资', "zi"),
            ('边', "bian"),
            ('流', "liu"),
            ('路', "lu"),
            ('级', "ji"),
            ('少', "shao"),
            ('图', "tu"),
            ('山', "shan"),
            ('统', "tong"),
            ('接', "jie"),
            ('知', "zhi"),
            ('较', "jiao"),
            ('将', "jiang"),
            ('组', "zu"),
            ('见', "jian"),
            ('计', "ji"),
            ('别', "bie"),
            ('她', "ta"),
            ('手', "shou"),
            ('角', "jiao"),
            ('期', "qi"),
            ('根', "gen"),
            ('论', "lun"),
            ('运', "yun"),
            ('农', "nong"),
            ('指', "zhi"),
            ('几', "ji"),
            ('九', "jiu"),
            ('区', "qu"),
            ('强', "qiang"),
            ('放', "fang"),
            ('决', "jue"),
            ('西', "xi"),
            ('被', "bei"),
            ('干', "gan"),
            ('做', "zuo"),
            ('必', "bi"),
            ('战', "zhan"),
            ('先', "xian"),
            ('回', "hui"),
            ('则', "ze"),
            ('任', "ren"),
            ('取', "qu"),
            ('据', "ju"),
            ('处', "chu"),
            ('队', "dui"),
            ('南', "nan"),
            ('给', "gei"),
            ('色', "se"),
            ('光', "guang"),
            ('门', "men"),
            ('即', "ji"),
            ('保', "bao"),
            ('治', "zhi"),
            ('北', "bei"),
            ('造', "zao"),
            ('百', "bai"),
            ('规', "gui"),
            ('热', "re"),
            ('领', "ling"),
            ('七', "qi"),
            ('海', "hai"),
            ('口', "kou"),
            ('东', "dong"),
            ('导', "dao"),
            ('器', "qi"),
            ('压', "ya"),
            ('志', "zhi"),
            ('世', "shi"),
            ('金', "jin"),
            ('增', "zeng"),
            ('争', "zheng"),
            ('济', "ji"),
            ('阶', "jie"),
            ('油', "you"),
            ('思', "si"),
            ('术', "shu"),
            ('极', "ji"),
            ('交', "jiao"),
            ('受', "shou"),
            ('联', "lian"),
            ('什', "shen"),
            ('认', "ren"),
            ('六', "liu"),
            ('共', "gong"),
            ('权', "quan"),
            ('收', "shou"),
            ('证', "zheng"),
            ('改', "gai"),
            ('清', "qing"),
            ('美', "mei"),
            ('再', "zai"),
            ('采', "cai"),
            ('转', "zhuan"),
            ('更', "geng"),
            ('单', "dan"),
            ('风', "feng"),
            ('切', "qie"),
            ('打', "da"),
            ('白', "bai"),
            ('教', "jiao"),
            ('速', "su"),
            ('花', "hua"),
            ('带', "dai"),
            ('安', "an"),
            ('场', "chang"),
            ('身', "shen"),
            ('车', "che"),
            ('例', "li"),
            ('真', "zhen"),
            ('务', "wu"),
            ('具', "ju"),
            ('万', "wan"),
            ('每', "mei"),
            ('目', "mu"),
            ('至', "zhi"),
            ('达', "da"),
            ('走', "zou"),
            ('积', "ji"),
            ('示', "shi"),
            ('议', "yi"),
            ('声', "sheng"),
            ('报', "bao"),
            ('斗', "dou"),
            ('完', "wan"),
            ('类', "lei"),
            ('离', "li"),
            ('华', "hua"),
            ('名', "ming"),
            ('确', "que"),
            ('才', "cai"),
            ('科', "ke"),
            ('张', "zhang"),
            ('信', "xin"),
            ('马', "ma"),
            ('节', "jie"),
            ('话', "hua"),
            ('米', "mi"),
            ('整', "zheng"),
            ('空', "kong"),
            ('元', "yuan"),
            ('况', "kuang"),
            ('今', "jin"),
            ('集', "ji"),
            ('温', "wen"),
            ('传', "chuan"),
            ('土', "tu"),
            ('许', "xu"),
            ('步', "bu"),
            ('群', "qun"),
            ('广', "guang"),
            ('石', "shi"),
            ('记', "ji"),
            ('需', "xu"),
            ('段', "duan"),
            ('研', "yan"),
            ('界', "jie"),
            ('拉', "la"),
            ('林', "lin"),
            ('律', "lv"),
            ('叫', "jiao"),
            ('且', "qie"),
            ('究', "jiu"),
            ('观', "guan"),
            ('越', "yue"),
            ('织', "zhi"),
            ('装', "zhuang"),
            ('影', "ying"),
            ('算', "suan"),
            ('低', "di"),
            ('持', "chi"),
            ('音', "yin"),
            ('众', "zhong"),
            ('书', "shu"),
            ('布', "bu"),
            ('复', "fu"),
            ('容', "rong"),
            ('儿', "er"),
            ('须', "xu"),
            ('际', "ji"),
            ('商', "shang"),
            ('非', "fei"),
            ('验', "yan"),
            ('连', "lian"),
            ('断', "duan"),
            ('深', "shen"),
            ('难', "nan"),
            ('近', "jin"),
            ('矿', "kuang"),
            ('千', "qian"),
            ('周', "zhou"),
            ('委', "wei"),
            ('素', "su"),
            ('技', "ji"),
            ('备', "bei"),
            ('半', "ban"),
            ('办', "ban"),
            ('青', "qing"),
            ('省', "sheng"),
            ('列', "lie"),
            ('习', "xi"),
            ('响', "xiang"),
            ('约', "yue"),
            ('支', "zhi"),
            ('般', "ban"),
            ('史', "shi"),
            ('感', "gan"),
            ('劳', "lao"),
            ('便', "bian"),
            ('团', "tuan"),
            ('往', "wang"),
            ('酸', "suan"),
            ('历', "li"),
            ('市', "shi"),
            ('克', "ke"),
            ('何', "he"),
            ('除', "chu"),
            ('消', "xiao"),
            ('构', "gou"),
            ('府', "fu"),
            ('称', "cheng"),
            ('太', "tai"),
            ('准', "zhun"),
            ('精', "jing"),
            ('值', "zhi"),
            ('号', "hao"),
            ('率', "lv"),
            ('族', "zu"),
            ('维', "wei"),
            ('划', "hua"),
            ('选', "xuan"),
            ('标', "biao"),
            ('写', "xie"),
            ('存', "cun"),
            ('候', "hou"),
            ('毛', "mao"),
            ('亲', "qin"),
            ('快', "kuai"),
            ('效', "xiao"),
            ('斯', "si"),
            ('院', "yuan"),
            ('查', "cha"),
            ('江', "jiang"),
            ('型', "xing"),
            ('眼', "yan"),
            ('王', "wang"),
            ('按', "an"),
            ('格', "ge"),
            ('养', "yang"),
            ('易', "yi"),
            ('置', "zhi"),
            ('派', "pai"),
            ('层', "ceng"),
            ('片', "pian"),
            ('始', "shi"),
            ('却', "que"),
            ('专', "zhuan"),
            ('状', "zhuang"),
            ('育', "yu"),
            ('厂', "chang"),
            ('京', "jing"),
            ('识', "shi"),
            ('适', "shi"),
            ('属', "shu"),
            ('圆', "yuan"),
            ('包', "bao"),
            ('火', "huo"),
            ('住', "zhu"),
            ('调', "diao"),
            ('满', "man"),
            ('县', "xian"),
            ('局', "ju"),
            ('照', "zhao"),
            ('参', "can"),
            ('红', "hong"),
            ('细', "xi"),
            ('引', "yin"),
            ('听', "ting"),
            ('该', "gai"),
            ('铁', "tie"),
            ('价', "jia"),
            ('严', "yan"),
            ('首', "shou"),
            ('底', "di"),
            ('液', "ye"),
            ('官', "guan"),
            ('德', "de"),
            ('随', "sui"),
            ('病', "bing"),
            ('苏', "su"),
            ('失', "shi"),
            ('尔', "er"),
            ('死', "si"),
            ('讲', "jiang"),
            ('配', "pei"),
            ('女', "nv"),
            ('黄', "huang"),
            ('推', "tui"),
            ('显', "xian"),
            ('谈', "tan"),
            ('罪', "zui"),
            ('神', "shen"),
            ('艺', "yi"),
            ('呢', "ne"),
            ('席', "xi"),
            ('含', "han"),
            ('企', "qi"),
            ('望', "wang"),
            ('密', "mi"),
            ('批', "pi"),
            ('营', "ying"),
            ('项', "xiang"),
            ('防', "fang"),
            ('举', "ju"),
            ('球', "qiu"),
            ('英', "ying"),
            ('氧', "yang"),
            ('势', "shi"),
            ('告', "gao"),
            ('李', "li"),
            ('台', "tai"),
            ('落', "luo"),
            ('木', "mu"),
            ('帮', "bang"),
            ('轮', "lun"),
            ('破', "po"),
            ('亚', "ya"),
            ('师', "shi"),
            ('围', "wei"),
            ('注', "zhu"),
            ('远', "yuan"),
            ('字', "zi"),
            ('材', "cai"),
            ('排', "pai"),
            ('供', "gong"),
            ('河', "he"),
            ('态', "tai"),
            ('封', "feng"),
            ('另', "ling"),
            ('施', "shi"),
            ('减', "jian"),
            ('树', "shu"),
            ('溶', "rong"),
            ('怎', "zen"),
            ('止', "zhi"),
            ('案', "an"),
            ('言', "yan"),
            ('士', "shi"),
            ('均', "jun"),
            ('武', "wu"),
            ('固', "gu"),
            ('叶', "ye"),
            ('鱼', "yu"),
            ('波', "bo"),
            ('视', "shi"),
            ('仅', "jin"),
            ('费', "fei"),
            ('紧', "jin"),
            ('爱', "ai"),
            ('左', "zuo"),
            ('章', "zhang"),
            ('早', "zao"),
            ('朝', "chao"),
            ('害', "hai"),
            ('续', "xu"),
            ('轻', "qing"),
            ('服', "fu"),
            ('试', "shi"),
            ('食', "shi"),
            ('充', "chong"),
            ('兵', "bing"),
            ('源', "yuan"),
            ('判', "pan"),
            ('护', "hu"),
            ('司', "si"),
            ('足', "zu"),
            ('某', "mou"),
            ('练', "lian"),
            ('差', "cha"),
            ('致', "zhi"),
            ('板', "ban"),
            ('田', "tian"),
            ('降', "jiang"),
            ('黑', "hei"),
            ('犯', "fan"),
            ('负', "fu"),
            ('击', "ji"),
            ('范', "fan"),
            ('继', "ji"),
            ('兴', "xing"),
            ('似', "si"),
            ('余', "yu"),
            ('坚', "jian"),
            ('曲', "qu"),
            ('修', "xiu"),
            ('故', "gu"),
            ('城', "cheng"),
            ('夫', "fu"),
            ('够', "gou"),
            ('送', "song"),
            ('笔', "bi"),
            ('船', "chuan"),
            ('占', "zhan"),
            ('右', "you"),
            ('财', "cai"),
            ('吃', "chi"),
            ('富', "fu"),
            ('春', "chun"),
            ('职', "zhi"),
            ('觉', "jue"),
            ('汉', "han"),
            ('画', "hua"),
            ('功', "gong"),
            ('巴', "ba"),
            ('跟', "gen"),
            ('虽', "sui"),
            ('杂', "za"),
            ('飞', "fei"),
            ('检', "jian"),
            ('吸', "xi"),
            ('助', "zhu"),
            ('升', "sheng"),
            ('阳', "yang"),
            ('互', "hu"),
            ('初', "chu"),
            ('创', "chuang"),
            ('抗', "kang"),
            ('考', "kao"),
            ('投', "tou"),
            ('坏', "huai"),
            ('策', "ce"),
            ('古', "gu"),
            ('径', "jing"),
            ('换', "huan"),
            ('未', "wei"),
            ('跑', "pao"),
            ('留', "liu"),
            ('钢', "gang"),
            ('曾', "zeng"),
            ('端', "duan"),
            ('责', "ze"),
            ('站', "zhan"),
        ].iter().cloned().collect();
        
        char_to_pinyin.get(&ch).map(|s| s.to_string())
    }

    /// 拼音转音素（简化版）
    fn pinyin_to_phonemes(&self, pinyin: &str) -> Vec<String> {
        // 这是一个非常简化的实现
        // 实际应该使用完整的拼音到音素转换规则
        
        let mut phonemes = Vec::new();
        let pinyin_lower = pinyin.to_lowercase();
        
        // 尝试直接查找拼音是否在 phone_id_map 中
        if self.phone_to_id.contains_key(&pinyin_lower) {
            phonemes.push(pinyin_lower);
            return phonemes;
        }
        
        // 如果找不到，尝试将拼音拆分为声母和韵母
        // 这里使用简化的规则：假设拼音是声母+韵母的组合
        // 实际应该使用更复杂的拼音分析
        
        // 常见声母
        let initials = ["b", "p", "m", "f", "d", "t", "n", "l", "g", "k", "h", 
                       "j", "q", "x", "zh", "ch", "sh", "r", "z", "c", "s"];
        
        // 尝试匹配声母
        for initial in &initials {
            if pinyin_lower.starts_with(initial) {
                if self.phone_to_id.contains_key(*initial) {
                    phonemes.push(initial.to_string());
                }
                // 提取韵母部分
                let remainder = &pinyin_lower[initial.len()..];
                if !remainder.is_empty() && self.phone_to_id.contains_key(remainder) {
                    phonemes.push(remainder.to_string());
                }
                break;
            }
        }
        
        // 如果没有找到匹配，尝试整个拼音作为音素
        if phonemes.is_empty() {
            if self.phone_to_id.contains_key(&pinyin_lower) {
                phonemes.push(pinyin_lower);
            } else {
                // 如果还是找不到，使用 <unk>
                phonemes.push("<unk>".to_string());
            }
        }
        
        phonemes
    }

    /// 将音素序列转换为音素 ID 序列
    pub fn phonemes_to_ids(&self, phonemes: &[String]) -> Result<Vec<i64>> {
        let mut ids = Vec::new();
        
        for phone in phonemes {
            // 尝试直接查找
            if let Some(&id) = self.phone_to_id.get(phone) {
                ids.push(id);
            } else {
                // 如果找不到，尝试查找小写版本
                let phone_lower = phone.to_lowercase();
                if let Some(&id) = self.phone_to_id.get(&phone_lower) {
                    ids.push(id);
                } else {
                    // 如果还是找不到，使用 <unk> 的 ID（通常是 1）
                    let unk_id = self.phone_to_id.get("<unk>")
                        .copied()
                        .unwrap_or(1);
                    ids.push(unk_id);
                }
            }
        }
        
        Ok(ids)
    }

    /// 将文本直接转换为音素 ID 序列（便捷方法）
    pub fn text_to_phone_ids(&self, text: &str) -> Result<Vec<i64>> {
        let phonemes = self.text_to_phonemes(text)?;
        self.phonemes_to_ids(&phonemes)
    }

    /// 获取音素 ID 映射（用于调试）
    pub fn get_phone_to_id_map(&self) -> &HashMap<String, i64> {
        &self.phone_to_id
    }
}

