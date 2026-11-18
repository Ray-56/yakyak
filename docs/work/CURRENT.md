# 当前工作

2026-11-18 AC-01-01 收尾归档。验收报告、进度和任务记录随本次文档提交归档，实现提交为 `f604dd50bfc5eb1afceb03ec133f43997720f5e7`，人工审阅结果为 `passed`。验收测试沿用该实现提交的独立干净检出结果。源码和协议测试未改动，本轮未重跑功能测试。下一项是 AC-01-02 字节边界，状态为 `not_started`。

2026-11-18 后续目标分析：AC-01-01 已通过，验收文档待提交。下一项为 AC-01-02 字节边界。现有解析已保留原始 body，但尚未核对 Content-Length。建议先为单条消息补一个截断测试，确认声明长度大于实际字节数时返回明确错误，再在 `parse_sip_message` 解析头字段后、复制 body 前加入检查。后续覆盖 UTF-8 字节计数、超长与冲突长度，复用空 body 和含换行的二进制正例。TCP 累计缓冲和分帧见 AC-10-01。本轮只分析并记录，未修改源码或运行协议测试，AC-01-02 仍为 `not_started`。

2026-11-18 AC-01-01已获用户整项审阅通过：用户在本会话明确确认“已审阅通过”。据此将AC-01-01从ready_for_review登记为passed，审阅者记录为用户（本会话明确确认），reviewed_at为2026-11-18（客户端Asia/Shanghai日期），不虚构额外身份或确认。实现提交仍为f604dd50bfc5eb1afceb03ec133f43997720f5e7，证据报告docs/acceptance/ch01/AC-01-01.md和独立clean复验日志artifacts/ac01-01/20261118-commit-recheck-tuqrttvd保持绑定，来源仍为实跑bash scripts/check.sh退出0（21 Python+135 Rust、fmt、严格Clippy、release通过）。本轮仅更新报告、进度和工作记录；源码哈希与被测提交一致，未重跑协议/CLI测试，未创建新提交或push。相邻02/03/04范围及既有未运行项保持原约定，未启动新AC实现。下一步由用户安排后续目标；验收文档和进度改动仍待提交。

2026-11-18 实现已提交并按固定SHA复验待审：用户明确要求代理提交实现。代理仅提交src/lib.rs与src/sip.rs，生成f604dd50bfc5eb1afceb03ec133f43997720f5e7（feat(sip): add message and URI parsing and serialization），2文件/2500行新增；其余文档及产物未混入，未push。提交内三份源码与前轮已通过检查的快照逐字一致，独立只读代理复核一致。已在该SHA的独立detached clean worktree实跑bash scripts/check.sh，退出0：21 Python、127协议+2CLI+6集成共135 Rust测试通过（0 failed/ignored/filtered），fmt、严格Clippy及release通过；检查前后Git状态为空，源码哈希未变，主工作区源码也与提交一致。证据目录artifacts/ac01-01/20261118-commit-recheck-tuqrttvd，含check.log与manifest.json，独立检出保留在忽略目录供追溯。验收报告docs/acceptance/ch01/AC-01-01.md已绑定完整SHA，区分干净被测检出与主工作区未提交文档。进度登记ready_for_review、implementation_commit真实SHA、evidence指向报告；reviewer/reviewed_at仍为null，不冒签人工确认，不标passed。当前授权仅提交两个实现文件，文档状态/报告/工作记录保留未提交；下一步用户审阅报告并明确整项验收结论。边界仍为有限合法样例，完整长度矩阵归02、不可信输入归03、完整语义/独立工具归04，远端CI/网络互通未运行。代理未改协议逻辑或测试，仅本地提交并更新验收及工作记录。

2026-11-17 字段公开复核与验收草稿完成：用户已补齐五结构体17字段pub，六模型、三个入口及Header::matches_name公开且helper私有。独立子代理确认本轮只改字段可见性，127协议测试未变。bash scripts/check.sh实跑退出0：21 Python、127协议+2CLI+6集成共135 Rust测试通过（0 failed/ignored/filtered），fmt、严格Clippy及release构建全部通过；检查期间三份源码哈希未变。证据目录artifacts/ac01-01/20261117-field-api-final-check-q_vxgyhx，含完整日志、manifest环境/哈希、tracked.patch及未跟踪源码快照。已按模板新建docs/acceptance/ch01/AC-01-01.md验收草稿，明确本次对象为dirty工作区、基线d048f217f2ff2e79fef5ae57c9da5ed404064b10不是实现提交，记录T01-T08有限样例及02/03/04边界和未运行项。当前source尚未提交；下一步用户先提交src/lib.rs和src/sip.rs固定实现SHA，之后代理在固定提交上复验并补齐报告/登记待审。AGENTS及ADR0003要求真实可达实现提交、可复现检查、证据与人工整项确认；本轮不能标ready_for_review或passed。保持in_progress，implementation_commit=null、evidence=[]、reviewer/reviewed_at=null，未冒签人工审阅。代理未修改三份源码或协议测试，仅创建验收草稿、更新工作记录与本项备注，未提交或推送。

2026-11-17 完整检查通过与结构体字段待公开：用户已公开六模型、三个解析/序列化入口及Header::matches_name，其余helper仍私有。独立子代理与前轮备份比较，移除新增pub后源码逐字一致，127项协议测试未漏；但五个结构体的17个字段仍全部私有，尚未完成原计划字段公开。实跑bash scripts/check.sh退出0：文档validate/render通过、21项Python测试通过、fmt通过、Clippy -D warnings通过、127协议+2CLI+6集成共135项Rust测试通过（0 failed/ignored/filtered），release构建通过。日志artifacts/ac01-01/20261117-public-api-full-check-jhyi66zh/check.log。此结果来自dirty且有未跟踪源码的工作区，不是固定提交复验，不能据此标ready_for_review或passed。当前内部子模块单测可访问私有字段，因此全检查通过不证明库外可读取/构造模型。下一小步用户只在RequestLine、StatusLine、Header、SipMessage、SipUri的全部字段前加pub，保留已有derive、类型/方法/函数可见性和逻辑；以SipMessage的start_line/headers/body为例。StartLine enum变体已随pub enum公开。新字段修改尚未执行或验证；完成后再复跑检查。未来实现提交/证据报告固定前保持in_progress、implementation_commit=null、evidence=[]、reviewer/reviewed_at=null；本轮不创建正式验收报告。三份源码均已备份并记录SHA256，未追踪文件也纳入快照。代理未改源码或测试，仅更新工作记录与本项备注。

2026-11-17 库导入格式复核与公开接口下一步：用户已删除main.rs未使用的use yakyak::sip并规范格式，main现在与原CLI壳一致；lib.rs为pub mod sip且末尾换行正确。RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check退出0。RUSTUP_AUTO_INSTALL=0 cargo clippy --locked --offline --all-targets -- -D warnings退出101：unused_imports诊断已消失，library仍有23项dead_code；日志artifacts/ac01-01/20261117-library-api-review-u6t6qnvr/clippy.log。仅导入/格式整理，本轮不重复功能测试（0项执行），上轮全目标127协议+2CLI+6集成共135项通过仍为最近全量结果。主/子代理及Rust官方visibility参考核对下一小步：用户在sip.rs公开RequestLine、StatusLine、StartLine、Header、SipMessage、SipUri六模型，五个结构体全部字段也pub，pub enum的Request/Response变体自动公开无需另加pub；公开三个真实入口parse_sip_message、serialize_sip_message、parse_sip_uri，以及Header::matches_name，其他helper保持私有。此范围用于库调用方实际解析、构造、读取与头名查询，不关闭lint或伪造CLI调用；缺类型可见性会产生private_interfaces，缺字段可见性不利于调用方读取/构造。新可见性修改尚未执行或验证，当前合法且长度一致的序列化输入前提不变。下一步完成后用户cargo fmt并bash scripts/check.sh，代理再核对实际命令/数量；完整check.sh及release本轮未执行，正式验收与实现提交仍待完成。代理未改三份源码或协议测试，仅更新工作记录与本项备注；AC保持in_progress。

2026-11-17 同包库搬迁复核与导入格式待整理：用户已建立src/lib.rs（pub mod sip;）与src/sip.rs，搬入六协议模型、函数、HashSet及内部协议tests；main.rs保留CLI和两项CLI单测，没有重复mod sip声明。独立只读子代理与搬迁前备份artifacts/ac01-01/20261117-if-fixed-before-library-move-k3d8vz2h/src/main.rs逐行比较，协议实现/断言完全一致。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --all-targets退出0：library协议单测127、binary CLI单测2、bootstrap集成测试6，共135 passed、0 failed、0 ignored、0 filtered out，测试未漏搬或重复。普通library构建有23项dead_code警告；main.rs:4新增use yakyak::sip但尚未使用，另有unused_imports警告。RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check退出1：lib.rs缺文件末尾换行、main.rs多空行。下一小步用户删除main.rs未使用导入并运行cargo fmt --all，再整理库接口可见性；未要求为消除警告调用协议或关闭lint。协议项仍私有符合本步分阶段迁移，后续应公开真实三个入口、六模型必要字段与matches_name，其余helper私有。以后协议定向测试选择--lib，原--bin yakyak仅有两项CLI单测，筛选0项不能算通过。Clippy、完整check.sh和release本轮未重跑，不宣称全工程通过；全目标测试已真实执行，更新前轮未运行状态。代理未改三份源码或协议测试，仅更新工作记录和本项备注；AC保持in_progress，正式验收证据与固定实现提交仍未完成。

2026-11-17 嵌套if修正复核与同包库搬迁下一步：用户已用let chain合并password嵌套if，白名单、None短路及空密码允许行为与原实现等价。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri 退出0：85 passed、0 failed、0 ignored、44 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。RUSTUP_AUTO_INSTALL=0 cargo clippy --locked --offline --all-targets -- -D warnings 退出101：needless_return与collapsible_if均已消失，仅剩普通CLI的23项dead_code；日志artifacts/ac01-01/20261117-clippy-module-review-ov5k__q8/clippy.log。主/子代理及ADR-0001/Cargo官方布局核对：当前只有main.rs，下一小步由用户建立同包库lib.rs（pub mod sip;）与单sip.rs，搬六模型、Header impl、全部协议函数和127个协议单测及HashSet导入；main.rs保留CLI、OsString/ExitCode导入及两项CLI单测。Protocol tests只依赖协议符号，迁移后仍在sip.rs内部cfg(test)模块通过use super::*访问私有项。main.rs不再声明mod sip以避免双份编译，Cargo.toml无需改动。本步先保持函数逻辑和断言不变，暂不调整可见性；搬迁后用cargo test --locked --offline --all-targets验证测试数量，库的普通构建仍可能报dead_code，随后再逐步公开三个真实入口、六模型必要字段与matches_name。未提前创建空模块或修改源码；新文件、搬迁及迁移后检查均尚未执行。当前main.rs已逐字节备份，路径见本段后补充。完整check.sh及release本轮未执行，AC仍待工程收尾和正式验收证据。代理仅更新工作记录和本项备注，状态保持in_progress。 源码备份：artifacts/ac01-01/20261117-if-fixed-before-library-move-k3d8vz2h/src/main.rs。

2026-11-17 末尾return修正复核与嵌套if下一步：用户已将parse_sip_uri内URI headers map的None分支改为Err表达式，错误语义保持。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak rejects_sip_uri_with_header_without_equals 退出0：1 passed、0 failed、0 ignored、128 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。RUSTUP_AUTO_INSTALL=0 cargo clippy --locked --offline --all-targets -- -D warnings 退出101：needless_return诊断已消失，仍有23项普通CLI dead_code与一项collapsible_if（bin test仅后一项）；日志artifacts/ac01-01/20261117-clippy-if-review-pn9l5pi9/clippy.log。下一小步用户合并password的if let Some(pwd)与内部非法字符if，使用let chain：先匹配Some，再以&&判断!pwd.bytes().all原白名单，非法时返回原错误。主/子代理核对Cargo edition2024及固定/最低Rust1.96.0，Rust官方文档确认let chains可用；None时右侧短路，空密码all=true维持现有允许行为。新改法尚未写入或运行，聊天提供示例由用户修改，不新增测试、不关闭lint。之后处理同包库组织导致的dead_code与完整检查/正式验收。完整check.sh、全部bin单测、全目标测试与release本轮未执行。代理未改源码或测试，仅更新工作记录与本项备注；AC保持in_progress。

2026-11-17 头空白模型复核与Clippy末尾return下一步：用户已在parse_and_serialize_headers_with_whitespace补齐两个Header的完整有序字段断言，并保留规范化输出比较，写法正确；输出比较放在模型比较前同样有效。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_and_serialize_headers_with_whitespace 退出0：1 passed、0 failed、0 ignored、128 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。RUSTUP_AUTO_INSTALL=0 cargo clippy --locked --offline --all-targets -- -D warnings 退出101，仍有23项普通CLI dead_code及collapsible_if、needless_return两项风格诊断，bin test目标也报后两项；日志artifacts/ac01-01/20261117-clippy-tail-return-review-h_ebos11/clippy.log。下一小步用户只将parse_sip_uri内URI headers map的None分支改为None => Err(String::from("header 必须包含等号")),，去掉return和分号；这是闭包最后match的分支值，错误语义不变。主/子代理只读复核及Clippy官方needless_return说明一致，建议尚未写入或验证。随后处理password嵌套if。另一子代理完成只读工程组织预研：同Cargo包lib.rs+单sip.rs可容纳纯函数和协议单测，CLI与其两项测试留main；未来可公开parse_sip_message/serialize_sip_message/parse_sip_uri、六模型必要字段及Header::matches_name，其余helper私有。ADR-0001/0002允许此分工；本轮未迁移文件、修改可见性、关闭lint或伪造CLI协议调用。完整check.sh、全目标测试和release本轮未执行，AC仍待工程收尾、固定实现提交及正式验收证据。代理未改源码或测试，仅更新工作记录与本项备注；AC保持in_progress。

2026-11-17 头空白输出复核与模型断言待补：用户新增parse_and_serialize_headers_with_whitespace，使用冒号SP/HTAB、续行及两条同名未知头，手写规范化输出X-Trace-ID: AbC 123与X-Trace-ID: DeF（各带CRLF）正确。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak --quiet 退出0：129 passed、0 failed、0 ignored、0 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。主/子代理只读复核发现本测试只有序列化输出断言，尚未补原计划中的两个Header完整字段比较。本轮下一小步调整为先在let headers后、现有输出断言前，assert_eq!(headers,vec![Header{name:X-Trace-ID,value:AbC 123},Header{name:X-Trace-ID,value:DeF}])，明确条目数量、字段值与顺序；保留现有输出断言。该新增模型断言尚未写入或运行，静态预计通过。之后再处理Clippy：URI headers map中None分支末尾return Err可改为分支值Err，另有password嵌套if及仅测试使用导致的dead_code收尾；本轮未修改这些代码。前轮check.sh失败记录仍未复核，日志artifacts/ac01-01/20261117-baseline-review-aebmo1xz/check.log；本轮未重跑完整检查，全目标测试/release仍未执行。代理未改源码或测试，仅更新工作记录与本项备注；AC保持in_progress。

2026-11-17 扩展方法复核与头空白规范化下一步：用户新增parse_and_serialize_extension_method，完整比较X-PING Request的method/uri/version及serialize输出，写法正确。主/子代理只读复核一致。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_and_serialize 退出0：2 passed、0 failed、0 ignored、126 filtered out（扩展方法与183响应）；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。T01已补原计划的扩展方法、临时响应及含空格原因短语输出正例，不据此声称完整语法验证。下一小步用户新增parse_and_serialize_headers_with_whitespace：未知头X-Trace-ID的冒号前后含SP/HTAB，首值AbC后续行含123，再跟一条同名未知头值DeF；完整比较两个Header，值依次AbC 123与DeF，并比较手写规范化输出的两行X-Trace-ID: AbC 123和X-Trace-ID: DeF（各带CRLF，不含结束空行）。RFC3261 §7.3.1/§25.1已核对冒号空白和折行展开单SP规则；本例预期不直接使用带折行input。新测试尚未新增或运行，静态预计通过；之后处理工程检查与验收收尾。前轮完整check.sh的dead_code与两条风格lint失败尚待处理/复核，本轮未重跑全部bin单测或完整检查，后续全目标测试/release未执行的状态不变。代理未改源码或测试，仅更新工作记录与本项备注；AC保持in_progress。

2026-11-17 临时响应正例复核与扩展方法下一步：用户新增parse_and_serialize_provisional_response，完整比较183/Session Progress的Response模型，并比较serialize_start_line输出与人工输入，字段、内部空格和CRLF均正确。主/子代理只读复核一致。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_and_serialize_provisional_response 退出0：1 passed、0 failed、0 ignored、126 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。下一小步用户新增parse_and_serialize_extension_method，固定输入X-PING sip:bob@example.com SIP/2.0加CRLF；完整比较StartLine::Request(RequestLine{method:X-PING,uri:sip:bob@example.com,version:SIP/2.0})，再比较serialize_start_line输出==人工input，验证合法扩展方法原样保留。RFC3261 §25.1的extension-method=token规则已在上轮核对；新测试尚未新增或运行，当前代码静态预计通过，不据此声称非法method已全面校验。T06空白/折行规范化输出、Clippy收尾及正式验收证据仍待完成。前轮check.sh在23项普通CLI dead_code及两项风格lint失败，日志artifacts/ac01-01/20261117-baseline-review-aebmo1xz/check.log；本轮未重跑完整工程检查或全部bin单测，先前脚本后续全目标测试/release未执行的状态不变。代理未改源码或测试，仅更新工作记录和本项备注；AC保持in_progress。

2026-11-17 重复头输出复核与AC剩余项核查：用户已在Via/Contact混合头测试分别补serialize_headers输出==人工input，原完整Header字段和名称断言均保留，写法正确。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak --quiet 退出0：126 passed、0 failed、0 ignored、0 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。主/两位子代理只读对照原任务卡T01至T08：T01还缺合法扩展方法、临时响应和含空格原因短语输出的验收断言；T06还缺冒号SP/HTAB及折行展开后的手写规范化输出断言。T02现有有限URI样例、T03/T04原值/列表/参数保留、T05十组紧凑名及大小写、T07空body请求/hello响应/二进制输出、T08既有确定性枚举和初步错误已有对应测试；不声称完整RFC或列表结构化解析验收。运行bash scripts/check.sh退出101：文档validate/render、21项Python测试、fmt均通过，cargo clippy --locked --all-targets -- -D warnings失败。普通CLI构建中有23项协议模型/函数dead_code（目前只被测试使用），另有collapsible_if（src/main.rs:238）与needless_return（273）两项风格诊断；bin test目标同样报这两项风格诊断。日志存artifacts/ac01-01/20261117-baseline-review-aebmo1xz/check.log。脚本后续cargo test --all-targets及release构建未执行，不能记为全工程通过；未关闭lint或自动重构。下一小步用户新增parse_and_serialize_provisional_response，输入SIP/2.0 183 Session Progress加CRLF，完整比较StartLine::Response(StatusLine{version:SIP/2.0,status_code:183,reason_phrase:Session Progress})并断言serialize_start_line输出等于人工input，一条补临时响应与含空格原因输出。RFC3261 §25.1已核对183及extension-method规则；新测试尚未写入或运行，静态预计通过。之后补扩展方法、T06规范化输出并处理工程检查及验收证据。完整长度/截断矩阵留02、恶意输入/限额留03、完整语义矩阵/独立工具留04。代理未改源码或测试，仅更新任务卡、CURRENT及本项备注；AC保持in_progress。

2026-11-17 二进制序列化复核与重复头输出下一步：用户新增 serialize_sip_message_preserves_binary_body，响应模型和人工预期均为7头、CL6、application/octet-stream，原body六字节含非UTF8/NUL/CRLFCRLF，完整输出比较正确。主/子代理只读复核一致。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak serialize_sip_message 退出0：3 passed、0 failed、0 ignored、123 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。下一小步在现有 parse_headers_preserves_via_list_and_repeated_header 与 parse_headers_preserves_contact_list_and_repeated_header 末尾各加 assert_eq!(serialize_headers(&headers), input)，保留原有人工写定的完整Header字段断言及名称匹配断言。两个input均采用规范冒号SP和CRLF且无折行，适合逐字比较；原字段断言独立约束解析结果，因此不单凭往返相等判定正确，并节省重复fixture。此两条输出断言尚未新增或运行，静态预计通过；随后对照本AC核查剩余项。完整长度/截断矩阵仍归AC-01-02，完整语义往返矩阵/独立工具归04；更多语法与整项验收未完成。代理未改源码或测试，仅更新工作记录和本项备注；本轮未重跑全部bin单测或完整工程检查，AC保持in_progress。

2026-11-17 带body响应序列化复核与二进制下一步：用户新增 serialize_sip_message_with_response_and_body，直接构造200 OK响应、7头原序、CL5与hello，完整输出比较正确且无多余尾部换行。主/子代理只读复核一致。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak serialize_sip_message 退出0：2 passed、0 failed、0 ignored、123 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。下一步用户新增 serialize_sip_message_preserves_binary_body，沿用响应结构，Content-Type改application/octet-stream、CL6、body为b"\xff\0\r\n\r\n"六字节；预期头部人工写定，as_bytes().to_vec()后追加同一固定body并完整比较。此处to_vec用于得到能继续追加的缓冲区，区别于仅比较时的可省复制。新测试尚未新增或运行；当前实现静态预计通过。本步只验证正常长度一致模型的原始字节输出，完整长度/截断/冲突矩阵仍归AC-01-02，其他语法、重复头/参数输出及整项验收未完成。代理未改源码或测试，仅更新工作记录和本项备注；全部bin单测与完整工程检查本轮未重跑，AC保持in_progress。

2026-11-17 完整空body序列化复核与切片比较讲解：用户已实现 serialize_sip_message：复用两个文本helper、追加单个CRLF结束空行、into_bytes后extend_from_slice原样追加body，拼接边界正确；serialize_headers已直接返回String。新增serialize_sip_message_without_body，直接构造大写SIP/2.0、七头原序、CL0及空body模型，完整比较预先写定的OPTIONS字节。用户询问expected未to_vec为何仍可通过：expected为&str，as_bytes为&[u8]，返回为Vec<u8>；Rust标准库提供Vec<T>对&[U]的PartialEq，比较长度及元素，无需把切片复制成Vec，上一条示例的to_vec可省。已核对Rust官方slice/Vec PartialEq文档及alloc/vec/partial_eq.rs源码说明。完整消息新增组合路径后实跑 RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak --quiet 退出0：124 passed、0 failed、0 ignored、0 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。主/子代理只读复核后，下一小步用户新增serialize_sip_message_with_response_and_body，复用现有响应解析测试中手写200 OK模型与完整预期文本，保留7头、CL5、body b"hello".to_vec()，完整字节断言可直接比较expected.as_bytes()。该新增测试尚未写入或运行；现实现静态预计通过。二进制body输出、更多重复头/参数的输出断言、更多语法及完整AC验收仍待后续；普通SipMessage的发送合法性/CL一致性仍是当前窄契约的输入前提。代理未改源码或测试，仅更新工作记录与本项备注；完整工程check.sh/CLI集成等未运行，AC保持in_progress。

2026-11-17 头列表序列化修正复核与完整消息下一步：用户已将serialize_headers改为String::new()/push_str，去掉三个Header.value中的CRLF，并修正expected中的branch拼写；完整三头比较和空切片断言正确。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak serialize_headers_preserves_names_values_and_order 退出0：1 passed、0 failed、0 ignored、122 filtered out。RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0，上一轮格式问题已消失。末尾headers_str.to_string()仍可直接返回headers_str以免复制，代理仅建议不修改。主/子代理只读复核下一小步：用户新增fn serialize_sip_message(message:&SipMessage)->Vec<u8>，仅编码本AC合法且Content-Length与body字节长度一致的手写模型；先用String拼接serialize_start_line和serialize_headers，再追加一次CRLF作为头部结束空行，into_bytes转Vec后extend_from_slice原样追加body并返回。现有两个helper已经各带行尾，不能再多加起始行/各头的行尾。Rust官方into_bytes文档已核对消费String并转成Vec<u8>。新增serialize_sip_message_without_body，复用现有无body OPTIONS解析测试中手写起始行、七头及空body模型，完整断言预先写定的消息字节，CL0且SIP/2.0大写；模型直接构造，期望不用自己的解析器生成。范围复核确认此窄契约可先用Vec签名，完整CL/截断/冲突矩阵仍归AC-01-02；普通SipMessage并不强制合法或CL一致，不能据此宣称任意parse成功对象都满足发送前提（已有接收容忍小写版本，发送版本另有格式要求）。新函数与测试尚未新增或运行，完整消息/body序列化、其他语法与AC验收仍未完成。代理未改源码或测试，仅更新工作记录和本项备注；完整工程检查未运行，AC保持in_progress。

2026-11-17 头列表 concat 编译错误定位：用户新增 serialize_headers 与完整三头/空切片测试，询问 concat 报错。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak serialize_headers_preserves_names_values_and_order 退出101，E0599定位src/main.rs:663：headers是&[Header]，不满足[Header]:Concat<_>，编译失败，0项单测执行。现有定向测试已直接复现编译期症状，编译诊断足以定位，因此不另造复现、不加日志/探针、不为已确定的类型错误构造多假设；按diagnosing-bugs流程采用现成反馈入口。仓库未发现GLOSSARY.md，复读相关ADR-0002的I/O分离约定。主/子代理只读核对：headers_str初始化为空字面量故类型为&str，应改为String::new()；循环内保留format!，改向headers_str调用push_str(&header_str)，末尾直接返回headers_str。Rust官方String::push_str和concat!文档已核对，聊天提供最小函数示例，由用户写入。另静态发现三个Header.value尾部含CRLF，序列化又追加CRLF会重复，应去掉模型value中的行尾；第二条expected将branch误写brnahc，应改回branch。测试expected里的concat!拼字面量可继续使用。以上修复尚未写入或运行，不声明修复后通过；本轮未运行格式或完整工程检查。代理未改源码或测试，仅更新任务卡与CURRENT，下一步用户修正函数及两处样例问题后重跑原测试，AC保持in_progress。

2026-11-16 起始行序列化复核与头列表下一步：用户已实现 serialize_start_line(&StartLine)->String，match 两个变体，用 format! 正确输出请求/响应字段、单空格及末尾CRLF；借用模型未移动字段。新增 serialize_start_line_formats_request_and_response，直接构造合法OPTIONS请求和200 OK响应，完整比较文本。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak serialize_start_line_formats_request_and_response 退出0：1 passed、0 failed、0 ignored、121 filtered out。RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出1，仅请求分支排版不合rustfmt，建议用户 cargo fmt --all；代理未执行格式化。此前Via测试函数名已改回preserves，本轮未单独重跑该测试。主/子代理只读复核后，下一小步用户新增 fn serialize_headers(headers: &[Header])->String：建空String，按原序循环，逐条追加原name、冒号和一个SP、原value、CRLF；结束空行由后续完整消息函数负责。新增 serialize_headers_preserves_names_values_and_order，手动构造vIa/v/X-Trace-ID三条，带不同branch及大小写值AbC-123，完整比较三行预期，再在同一测试断言空切片输出空字符串。新函数与测试尚未新增或运行；完整消息和body序列化、其他语法与AC验收未完成。代理未改源码或测试，仅更新工作记录和本项备注；完整工程检查未运行，AC保持in_progress。

2026-11-16 Via 混合复核与起始行序列化下一步：用户新增 Via 单行列表加重复 v 头测试，实际函数名为 parse_headers_perserves_via_list_and_repeated_header（perserves 拼写）。完整比较两个 Header 原名、原序、三个成员原值及各自 branch/received 参数，并分别断言 matches_name("Via")，写法正确。首次 rg 按 preserves 名称查找退出1（无匹配）；首次 RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_headers_preserves_via_list_and_repeated_header 退出0但0 passed、121 filtered out，未视为通过。按实际名称重跑 RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_headers_perserves_via_list_and_repeated_header 退出0：1 passed、0 failed、0 ignored、120 filtered out。RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。主/子代理只读核对后，下一小步用户在 parse_start_line 附近新增 fn serialize_start_line(line: &StartLine) -> String，对现有合法模型 match Request/Response，用 format! 生成字段间单个SP和末尾CRLF；借用输入。新增一个 serialize_start_line_formats_request_and_response 测试，直接构造请求和响应模型，完整断言 OPTIONS sip:bob@example.com SIP/2.0 加CRLF 与 SIP/2.0 200 OK 加CRLF。RFC3261 §7.1/§7.2/§25.1 已核对输出格式；原因短语原文保留，状态码后的空格也应保留，不对整行 trim。序列化函数与测试尚未新增或运行，模型合法性校验和完整消息输出未据此验收。代理未改源码或测试，仅更新工作记录和本项进度备注；完整工程检查未运行，AC 保持 in_progress。

2026-11-16 Contact 列表复核与 Via 混合下一步：用户新增 parse_headers_preserves_contact_list_and_repeated_header，完整比较两个 Header 的原名 Contact/m、原序及原值：首条含两个联系地址，第二条含第三个；显示名、x-note 引号内逗号和各成员 transport/expires/q 参数均保留，两个 matches_name("Contact") 断言正确。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_headers_preserves_contact_list_and_repeated_header 退出0：1 passed、0 failed、0 ignored、119 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。主/子代理只读复核现有 Via 仅覆盖多行，下一小步用户新增 parse_headers_preserves_via_list_and_repeated_header：首条 Via 含 UDP first:5060 与 TCP second:5070 两个列表成员，第二条 v 含 UDP third:5080，各自带不同 z9hG4bK-branch，首成员再含 received=192.0.2.1。完整比较两个 Header 的原名、原值、顺序，分别验证 matches_name("Via")；原文模型保存两条头、三个Via成员。RFC3261 §20.42/§25.1 已核对列表、紧凑名及参数语法。新测试尚未新增或运行，静态预计通过；此后进入序列化小步，列表结构化语义及序列化尚未验收。代理未改源码或测试，仅更新任务卡与 CURRENT；完整工程检查未运行，AC 保持 in_progress。

2026-11-16 Contact 名称断言复核与列表下一步：用户已在完整结构比较后补 result.unwrap()，分别断言两个 Header 的 matches_name("Contact")，写法正确。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_headers_preserves_contacts_and_parameters 退出0：1 passed、0 failed、0 ignored、118 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。主/子代理只读核对后，下一小步用户新增 parse_headers_preserves_contact_list_and_repeated_header：REGISTER场景头片段，首条 Contact 含 desktop/mobile 两个列表成员，第二条 m 含 tablet；首成员保留带逗号显示名及 x-note="desk, primary" 扩展头参数，另保留 transport、expires、q。按当前原文模型完整比较两个 Header 的原名、原值和顺序，并验证二者均匹配 Contact；两个头条目保存三个联系地址。RFC3261 §25.1 已核对 Contact 列表及 generic-param 的 quoted-string 语法。新测试尚未写入或运行，静态预计现有通用头解析可通过；本步仅验证列表原文保留，结构化成员拆分及序列化未据此验收。代理未改源码或测试，仅更新任务卡与 CURRENT；完整工程检查未运行，AC 保持 in_progress。

2026-11-16 Contact 原文保留复核与名称断言待补：用户新增 parse_headers_preserves_contacts_and_parameters，完整比较两个 Header：Contact/m 原名、原序、含引号和逗号的显示名、URI 内 transport 与头参数 expires/q 均原样保留。输入与预期一致，moblie 域名虽不同于建议拼写但合法且不影响本例。RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_headers_preserves_contacts_and_parameters 退出0：1 passed、0 failed、0 ignored、118 filtered out；RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 退出0。主/子代理只读复核确认尚未补两条 matches_name("Contact") 断言；下一小步仅在原 assert_eq! 后加分号，再 result.unwrap()，分别断言两个 Header 匹配 Contact。新增断言尚未写入或运行，之后再覆盖单行列表/混合以及序列化。代理未改源码或测试，仅更新任务卡与 CURRENT；完整工程检查未运行，AC 保持 in_progress。

2026-11-16 响应样例修正复核与 Contact 下一步：用户已将响应Via输入/预期均修正为z9hG4bK-test，与此前请求一致。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_sip_message` 退出0：3 passed、0 failed、115 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出0。下一步用户新增 parse_headers_preserves_contacts_and_parameters，仅解析REGISTER场景两行头片段：Contact带显示名"Alice, Desktop"、URI中transport=tcp及头参数expires=60;q=0.8；m行含另一个URI的transport=udp、expires=120;q=0.5。完整比较两个Header的原name(Contact/m)、原value及顺序，另验证每条matches_name("Contact")为true。RFC3261 §20.10/§25已核对参数语法；本步仅验证通用头值原文保存，不引入注册业务或结构化Contact解析。新测试尚未新增或运行，当前通用实现静态预计通过；单行列表/混合、序列化等本AC验收仍待后续。代理未改源码或测试，完整工程检查未运行，状态保持in_progress。

2026-11-16 CSeq 全称与响应样例复核：用户询问CSeq全称并报告测试通过。已查RFC6873及RFC2543官方文本，CSeq为Command Sequence（命令序列号），仅引用全称、不套用旧版事务语义。实读响应测试完整比较200/OK、7头与hello body；`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_sip_message` 退出0：3 passed、0 failed、115 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出0。发现样例输入和Via预期均写为z9hG4bk-test（小写k），与前述请求z9hG4bK-test不同。主/子代理及RFC3261 §8.1.1.7/§8.2.6.2核对：正常响应应复制原Via，3261固定前缀为z9hG4bK。下一小步用户仅将测试输入与预期两处改回大写K；本轮不增加branch语义校验或事务逻辑。修正尚未执行或验证，后续继续Contact/参数保留和序列化。代理未改源码或测试，完整工程检查未运行，AC保持in_progress。

2026-11-16 CSeq 概念讲解：用户询问CSeq用途，本轮仅概念讲解。已查RFC3261 §8.1.1.5/§8.2.6.2/§12.2.1.1/§20.16：格式为十进制请求序号和请求方法，帮助识别/排序事务；请求方法须与请求行一致，响应复制请求的CSeq。相同Dialog的同一发送方向新普通请求递增，重传保持原CSeq，ACK/CANCEL复用所关联请求的序号并使用相应方法；不是全局唯一ID，事务匹配还涉及Via branch等。结合样例解释响应为何仍是CSeq:1 OPTIONS，当前AC继续保留原头值，不新增CSeq模型、事务或Dialog实现。0项单元测试执行，未运行格式或完整工程检查，本轮未核对响应正例的新增实施进度；下一步仍继续完整带body响应测试。代理未改源码或测试，仅更新工作记录，AC保持in_progress。

2026-11-16 完整二进制 body 复核与响应下一步：用户新增完整OPTIONS二进制body正例：头部包含Content-Type: application/octet-stream与Content-Length:6，追加原始6字节body，断言message.body与原字节一致，写法正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_sip_message` 退出0：2 passed、0 failed、115 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出0。复读任务卡范围与AC-01-02：完整Content-Length/截断/冲突长度边界矩阵属于02，本AC先使用长度一致正常样例。下一步用户新增 parse_sip_message_with_response_and_body：SIP/2.0 200 OK响应，Via、From(tag123)、To(tag456)、Call-ID、CSeq:1 OPTIONS、Content-Type:text/plain、Content-Length:5共7头，body为hello；完整比较StartLine::Response(StatusLine)、7头原序与字节body。现有分支静态预计通过，新测试尚未新增或运行。重复Contact/参数、更多语法和序列化等本AC验收仍待后续。代理未改源码或测试，完整工程检查未运行，状态保持in_progress。

2026-11-12 完整请求解析复核与字节 body 下一步：用户已修正窗口长度2、headers切片pos+2、To预期末尾>，并采用三条明确的CRLF/UTF8错误文案。完整无body OPTIONS起始行、7头及空body断言正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_sip_message` 退出0：1 passed、115 filtered out；因完整消息组合涉及多个既有解析函数，又运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak --quiet` 退出0：116 passed、0 failed、0 ignored、0 filtered out。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出0。下一步用户新增 parse_sip_message_preserves_binary_body，复用完整OPTIONS样例，加入Content-Type: application/octet-stream、CL6，头部字符串保留结束空行，转Vec字节后extend_from_slice追加body b"\xff\0\r\n\r\n"，解析成功后断言body==原body.to_vec；静态预计通过，新测试未新增或运行。结构解析初步完成，不代表Content-Length/必需头语义或更多头值语法、重复Contact/参数、响应和序列化验收完成。代理未改源码或测试，完整check.sh/CLI集成等未运行，AC保持in_progress。

2026-11-12 完整消息解析切片与预期复核：用户新增 parse_sip_message 与七头 OPTIONS 无body测试（实际测试名 parse_sip_message_withou_body），请求排错与错误文案评析。首次 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_sip_message` 退出101：0 passed、1 failed、115 filtered out，因 windows(4) 比较两字节CRLF始终不匹配。期间用户同步改为windows(2)，主/子代理复读确认；按最新文件重跑同命令仍退出101、1 failed、115 filtered，实际已变为头字段不能为空。剩余根因：headers从head[pos..]开始，带入起始行CRLF，应从pos+2开始；测试To预期漏末尾>，修复切分后还会导致断言差异。建议三条错误文案分别说明起始行缺CRLF、起始行非有效UTF8、头字段非有效UTF8；现有仅头部文本转换、起始行保留CRLF及body.to_vec思路可保留。初次格式检查退出0；当前修复建议未写入或验证，代理未修改源码或测试，完整工程检查未运行，AC未完成。

2026-11-12 空输入反例复核与完整消息下一步：用户已在缺分隔符测试内补空输入 b"" 的相同错误断言，位置与文案正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak split_sip_head_and_body` 退出 0：3 passed、0 failed、112 filtered out；因仅新增断言，测试函数数未变。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；SipMessage 尚未构造警告仍存在。下一步用户实现 `parse_sip_message(input: &[u8]) -> Result<SipMessage, String>`，先拆分、仅头部from_utf8、按第一个CRLF保留起始行CRLF并分出headers调用既有函数，body.to_vec构造模型；测试 parse_sip_message_without_body 用包含Via/Max-Forwards/From/To/Call-ID/CSeq/Content-Length七头的OPTIONS样例，完整比较起始行、头顺序及空body。新函数与测试尚未新增或运行；本步先组合结构，不代表Content-Length或必需头语义已校验，后续仍需重复Contact/参数、完整二进制body、响应和序列化验收。代理未修改源码或测试，完整工程检查未运行，AC 保持 in_progress。

2026-11-12 缺分隔符反例复核与空输入待补：用户新增 split_sip_head_and_body_rejects_missing_separator，头行齐全但缺结束空行的输入完整断言现有错误，写法正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak split_sip_head_and_body` 退出 0：3 passed、0 failed、112 filtered out；仍有 SipMessage 尚未构造警告。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。源码尚未包含此前安排的空输入 b"" 断言，本轮明确不将其记为已通过。下一小步用户在现有缺分隔符测试尾追加 split_sip_head_and_body(b"") 与相同 Err 的比较，以覆盖少于四字节的无窗口输入；静态预计通过，该新断言尚未运行，之后组合完整消息解析。代理未修改源码或测试，完整工程检查未运行，仅更新工作记录，AC 未完成。

2026-11-12 二进制 body 复核与缺分隔符下一步：用户新增 split_sip_head_and_body_preserves_binary_body，完整比较头部及六字节body，非UTF8/NUL及body内后续CRLFCRLF原样保留。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak split_sip_head_and_body` 退出 0：2 passed、0 failed、112 filtered out；仍有 SipMessage 尚未构造警告。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增 split_sip_head_and_body_rejects_missing_separator，同一函数分别输入 OPTIONS行、Content-Length:0但缺结束空行的字节串，以及空字节串 b""，均完整断言 Err(String::from("SIP 消息缺少头部结束空行"))。当前位置查找与ok_or_else静态预计通过，新错误断言尚未新增或运行；覆盖首个分隔符缺失和短输入的无窗口边界。之后组合完整消息解析；Content-Length和更多消息语法、Contact/参数及序列化仍未验收。代理未改源码或测试，完整工程检查未运行，仅更新工作记录。

2026-11-12 字节拆分正例复核与二进制下一步：用户已交换预期tuple为 (expected_head,expected_body)，拆分函数及正例静态复核正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak split_sip_head_and_body` 退出 0：1 passed、0 failed、112 filtered out；仍有 SipMessage 尚未构造警告。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增 `split_sip_head_and_body_preserves_binary_body`：输入 OPTIONS 行、Content-Length:6、结束空行及原始body b"\xff\0\r\n\r\n"；预期头部保留最后CRLF，body六字节原样返回。单例验证非UTF8与NUL不被文本转换、body中的第二个分隔符不被误用；现用position取首个，静态预计通过，新测试尚未新增或运行。helper仅分界，样例长度为6但不表示已有Content-Length校验；缺分隔符错误、完整消息解析与其余AC验收仍待后续。代理未改源码或测试，完整工程检查未运行，仅更新工作记录。

2026-11-12 拆分测试元组顺序定位：用户已完成 windows/position 的字节拆分，函数返回 (head,body) 正确；测试预期值变量类型正确，但 assert_eq 把元组顺序写成 (expected_body,expected_head)。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak split_sip_head_and_body` 退出 101：0 passed、1 failed、112 filtered out，实际头部在前、hello在后，预期顺序相反。编译有 SipMessage 尚未构造的 dead_code 警告，不是断言失败原因。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。最小指导只交换测试预期元组为 (expected_head,expected_body)，无需更改函数；建议未写入或验证，代理未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-12 字节数组与切片类型讲解：用户新增 split_sip_head_and_body 的 todo 函数和测试，预期仍为 Ok((b"aaa",b"aaa"))，询问类型与实现。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak split_sip_head_and_body` 退出 101，E0308：实际 Result<(&[u8],&[u8]),String> 与预期自行推导出的 Result<(&[u8;3],&[u8;3]),_> 不匹配；编译同时提示 input 未使用，0 项单测执行。会话指导显式给 expected_head/expected_body 标注 &[u8] 并写真实头部与hello预期；helper用 windows(4).position 查首 CRLFCRLF，ok_or_else 转错误并用 ? 取得位置，返回原输入的 [..pos+2] 和 [pos+4..] 两个借用切片。+2 保留头部最后CRLF，+4跳过空行，不解码body，不做Content-Length或完整SIP语法校验。主/子代理核对类型和切片边界，示例未写入或执行；格式与完整工程检查未运行，AC 未完成。代理仅更新工作记录，未修改源码或测试。

2026-11-11 消息模型复核与字节拆分下一步：用户已定义 SipMessage，派生 Debug/PartialEq/Eq，字段 StartLine、Vec<Header>、Vec<u8> 正确。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；仅数据结构变更，本轮未运行单元测试（0 项执行），未运行完整工程检查。下一步用户实现纯字节拆分 helper `split_sip_head_and_body(input: &[u8]) -> Result<(&[u8], &[u8]), String>`，查首个 CRLFCRLF；头部取到分隔起点+2（保留最后行CRLF），body从起点+4开始，缺分隔返回明确错误；只做拆分，不解码或校验body长度。正例 `split_sip_head_and_body_with_body` 输入 OPTIONS 请求行、Content-Length: 5、空行及hello，分别断言头部包含最后CRLF及body为hello。helper及测试尚未新增或运行，随后由用户组合已有解析函数；Content-Length、二进制body、重复Contact/参数、序列化等完整AC仍待验收。代理未修改源码或测试，仅更新工作记录。

2026-11-10 十组紧凑形式复核与消息模型下一步：用户已扩展 canonical_header_name 至 RFC3261 十组紧凑映射，并新增其余九组循环测试，双向、查询大小写和原名保留断言均正确；未知名原样回退保持。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：25 passed、0 failed、87 filtered out（14 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户只定义 SipMessage，派生 Debug/PartialEq/Eq，字段 start_line: StartLine、headers: Vec<Header>、body: Vec<u8>，整合已有模型并按原始字节保存 body；RFC3261 §7.4.1 已核对允许二进制 body。本步不新增解析函数，模型尚未新增或验证；之后在完整消息样例中覆盖重复 Contact/参数、body 和序列化等剩余验收。代理未改源码或测试，完整工程检查未运行，AC 保持 in_progress。

2026-11-10 紧凑形式循环测试讲解：用户询问循环测试如何写；实读当前源码尚未新增 compact_forms 测试，归一函数仍仅支持 Via/v。本轮讲解将九组 (完整名,紧凑名) 元组放数组，再由 for (full,short) 解构每一组；直接构造两个 Header 检查名称匹配这一方法，逐组验证双向、查询大小写及名称原文，断言加入当前组诊断。一个测试函数中运行九组数据，不是九个测试函数，首次断言失败会停止该函数。示例仅聊天提供，由用户写入；当前预计第一组 Call-ID/i 失败，未实际运行示例或任何单元测试（0 项执行），也未运行格式和完整工程检查。代理未修改源码或测试，仅更新工作记录，完整 AC 验收未完成。

2026-11-10 紧凑 Via 与未知名复核：用户已实现 canonical_header_name，仅 v 忽略大小写映射 Via，其余原样返回；matches_name 对两侧归一后忽略大小写比较。Via/v 双向、短名大写、未知 abc 与原文保留断言均正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：24 passed、0 failed、87 filtered out（13 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增循环测试 header_name_comparison_supports_compact_forms，按 RFC3261 补其余九对映射的预期：Call-ID/i、Contact/m、Content-Encoding/e、Content-Length/l、Content-Type/c、From/f、Subject/s、Supported/k、To/t；每组验证长短名双向、大写短名及原文保留。当前仅实现 Via/v，其他长短名匹配预计失败；新测试尚未新增或运行，随后再由用户扩展现有归一函数，不逐别名单独建立任务。代理未改源码或测试，完整工程检查未运行，完整 AC 验收未完成。

2026-11-10 紧凑 Via 与未知头名回退讲解：用户新增 Via/v 双向及名称原文测试，matches_name 写到 lowercase 后的空 if，询问未知名 abc 如何处理。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header_name_comparison` 退出 101，E0308：方法要求 bool，空 if 产出 ()；没有测试执行。讲解方案为 canonical_header_name(name:&str)->&str，仅将忽略大小写的 v 映射为 Via，其余原样返回；存储名与查询名均归一后 eq_ignore_ascii_case，保留原字段并支持未知名 abc/ABC 匹配、abc/Via 不匹配。建议在现有测试添加 abc 正反与原文断言。方案只会话示例，未写入或验证；不将所有未知名称映射 Via，也不改变原始字段值。代理未修改源码或测试，本轮未运行格式或完整工程检查，其他紧凑别名及完整 AC 验收待后续。

2026-11-10 头名原文断言与 Via 紧凑形式下一步：用户已在大小写比较测试中新增原文 name=vIa 断言，并用 assert! 检查 Contact 不匹配，修改正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：23 passed、0 failed、87 filtered out；新增断言位于原函数内，测试函数数量不变。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 1，测试中空行有尾随空格，建议用户 cargo fmt --all。下一步用户新增 `header_name_comparison_supports_compact_via`，验证存储 v 时查询 Via/VIA/v/V 均匹配，存储 Via 时查询 v/V 也匹配，Contact 不匹配，原始名称保留。RFC3261 §20.42/§25.1 已核对 Via 紧凑形式为 v。当前仅 eq_ignore_ascii_case，跨长短名称预计失败，测试尚未新增或运行。本小步支持范围仅 Via/v，后续紧凑别名和完整 AC 待验收。代理未改源码或测试，完整工程检查未运行。

2026-11-10 头名大小写比较复核：用户新增 Header::matches_name，使用 eq_ignore_ascii_case，方法实现正确。新增 header_name_comparison_is_case_insensitive，从 vIa 头解析获得 Header，断言 vIa/Via/VIA/via 匹配以及 Contact 不匹配；成功用例中使用 unwrap 合理。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：23 passed、0 failed、87 filtered out（12 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。建议补 assert_eq!(header.name,"vIa") 验证保留名称原文；Contact false 的 assert_eq 写法正确，可按习惯简化为 assert!(!header.matches_name("Contact"))。本轮仅评析，未改源码或测试，新增原文断言尚未运行；紧凑别名及其他完整 AC 验收待后续，完整工程检查未运行。

2026-11-10 孤立续行复核与头名比较下一步：用户新增 rejects_headers_with_orphan_continuation，SP/HTAB 孤立续行分别完整断言相同错误，写法正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：22 passed、0 failed、87 filtered out（11 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户在 Header 的 impl 中新增 matches_name(&self,name:&str)->bool，使用 self.name.eq_ignore_ascii_case(name)，保留 name 原文；测试 header_name_comparison_is_case_insensitive，从 vIa 头解析得到对象，Via/VIA/via 匹配，Contact 不匹配，原 name 仍为 vIa。RFC3261 §7.3.1 已核对头名比较忽略大小写；紧凑别名下一步单独处理。方法和测试尚未新增或运行，示例仅会话讲解。代理未改源码或测试，完整工程检查未运行，Contact/参数、紧凑形式、消息/body、序列化等完整验收未完成。

2026-11-10 多续行与后续头组合复核：用户新增 parse_headers_preserves_header_after_fold，完整断言 Subject=hello world there 与后续未知头 X-Trace-ID=abc-123 的顺序和内容；SP/HTAB 多续行及切换下一头均正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：21 passed、0 failed、87 filtered out（10 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增 `rejects_headers_with_orphan_continuation`，同一函数分别调用 parse_headers，输入 Rust 字符串 `" world\r\n"` 与 `"\tworld\r\n"`，都完整比较 `Err(String::from("续行前缺少头字段"))`。现有 current 为空分支预计拒绝，测试尚未新增或运行；验证输入开头没有前置头时 SP/HTAB 不能凭空作为续行。其他错误边界、Contact/参数、紧凑形式、消息/body 和序列化验收待后续。代理未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 折行及最后逻辑头复核：用户已补循环结束后 current 非空时追加 CRLF 并解析 push，位置正确；折行及重复 Via 两个既有失败测试转绿。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：20 passed、0 failed、87 filtered out（9 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增组合正例 `parse_headers_preserves_header_after_fold`，输入 Rust 字符串 `"Subject: hello\r\n world\r\n\tthere\r\nX-Trace-ID: abc-123\r\n"`，完整断言依序为 Subject=hello world there、X-Trace-ID=abc-123，验证多续行、HTAB、结束折行后下一头不串入且未知头原文保留；静态预计通过，新测试尚未新增或运行。孤立续行、缺 CRLF 等错误路径及 Contact/参数、紧凑形式、消息/body、序列化仍待验收。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 循环后漏保存复核：用户已删除循环内旧的逐行 parse_header/push，当前仍未添加循环外最后 current 的保存块。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 101：18 passed、2 failed、87 filtered out；失败输出已变化：折行测试返回 Ok([])，重复 Via 测试只返回首条，直接印证最后逻辑头未提交。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步仅需用户在 for 结束之后、Ok(headers) 之前添加 current 非空时补 CRLF 并解析 push 的三行块；示例明确展示相邻右花括号和插入位置，未代改源码。当前两个小测试已充分定位，不添加日志或新测试；建议修改尚未验证。代理仅更新工作记录，完整工程检查未运行，AC 未完成。

2026-11-09 折行实现失败定位：用户按逻辑头缓冲方式修改 parse_headers 后报告单测失败。运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 101：18 passed、2 failed、87 filtered out；折行正例仍报缺冒号，重复 Via 实际首条保存两次形成三条。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。已复现并由主/子代理只读核对根因：循环末尾还保留旧 parse_header(line) 和 push 两行，同时缺少循环结束后提交 current 的块。最小指导是删除旧两行，在 for 外、Ok(headers) 前补 current 非空时追加 CRLF 并解析保存最后一条；其余合并逻辑可保留。两个现有输入已为直接最小场景，未再缩小、未添加探针或创建测试；未执行建议修复，不能声明已通过。已应用 diagnosing-bugs 技能，遵守用户亲写核心的授权范围；代理仅更新工作记录，未修改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 头字段折行缓冲讲解：用户新增折行正例，实际函数名为 parse_header_with_folded_value，询问实现思路。按此前建议复数名过滤的首次命令执行 0 项、107 项过滤，不算通过；随后按实际名运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_header_with_folded_value` 退出 101：0 passed、1 failed、106 filtered out，实际缺冒号错误。教学方案：以 String current 暂存不带 CRLF 的逻辑头；逐物理行验证 CRLF 和非空；SP/HTAB 续行追加单空格及仅 trim_start 后内容，孤立续行报错；新头到来时先提交前一 current（补 CRLF 调用 parse_header），再保存新头；新头第一物理行需包含冒号，避免非法跨行头名；循环后提交最后一条。示例仅在聊天提供，尚未写入或执行。代理未修改源码或测试，未运行格式或完整工程检查，完整 AC 未完成。

2026-11-09 重复 Via 复核与折行下一步：用户已实现 parse_headers：split_inclusive 保留 CRLF，逐行调用 parse_header 并按序 push，错误通过 ? 传播；重复 Via 测试完整断言两个不同值及顺序。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：19 passed、0 failed、87 filtered out（8 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增 `parse_headers_with_folded_value`，输入 Rust 字符串 `"Subject: hello\r\n world\r\n"`，期望一个 Subject=hello world。RFC 3261 §7.3.1 规定续行以 SP/HTAB 开头，换行及续行开头空白按一个空格解释；当前第二行单独解析，静态预计报缺冒号，新测试尚未新增或运行。后续应在 parse_headers 合并逻辑头再调用单行 parse_header，首行孤立续行需报错。未知字段、Contact/参数、紧凑形式、消息/body、序列化等仍需后续验收。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 单行头换行检查与多头下一步：用户将值检查改为 value.contains([NUL,CR,LF])，裸 LF 和裸 CR 两个断言均通过，原 NUL 反例保持通过。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：18 passed、0 failed、87 filtered out（7 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户实现 `parse_headers(input: &str) -> Result<Vec<Header>, String>`，本小步输入仅含每条以 CRLF 结尾的头字段，不含结束空行或 body，使用 split_inclusive 保留各行 CRLF、逐行调用 parse_header 并按原序 push。首个测试 `parse_headers_preserves_repeated_via` 使用两个不同值的 Via，完整比较两个 Header 的原顺序和内容；该函数与测试尚未新增或运行。折行、专用头值语法、未知头/紧凑形式、消息/body 和序列化等仍待后续。代理未改源码或测试；完整工程检查未运行，AC 保持 in_progress。

2026-11-09 头字段值扫描讲解：用户新增 rejects_header_with_embedded_line_breaks，包含裸 LF 和裸 CR 两个断言，询问是否需 bytes().all。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak rejects_header_with_embedded_line_breaks` 退出 101：0 passed、1 failed、104 filtered out；第一条 LF 断言实际返回 Ok，第二条 CR 断言因首断言 panic 未执行。讲解 any 判断是否存在禁止字节、all 判断是否全部满足规则；当前有限集合可直接用 value.contains([NUL,CR,LF]) 替换原 NUL 检查，或 bytes().any 检查三字节集合。合法折行和其他专用语法仍待后续，不扩大为完整头值验证。建议代码未写入或验证；本轮未运行格式和完整工程检查，代理仅更新工作记录，AC 未完成。

2026-11-09 头字段值 NUL 反例复核：用户新增 Subject 值中 NUL 的反例并用 value.contains(NUL) 拒绝，当前目标已通过。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：17 passed、0 failed、87 filtered out（6 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。目前只检查 NUL，其他值语法尚未完成。下一步用户新增 `rejects_header_with_embedded_line_breaks`，分别用 Rust 字符串 `"Subject: hello\nworld\r\n"` 和 `"Subject: hello\rworld\r\n"`，期望 `Err(String::from("头字段值包含非法控制字符"))`；去掉末尾 CRLF 后，值中仍有裸 LF/CR，现实现静态预计接受，新反例尚未新增或运行。合法折行需单独处理，不能将此步表述为禁止 SIP 头的所有折行；quoted-pair 也有专用语法，不推荐简单将所有字段值一律拒绝所有控制字节。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 头名 token 检查复核：用户已用完整 SIP token 字符白名单替换仅查空格的判断，保留名称非空检查，并新增 Bad@Name 反例。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：16 passed、0 failed、87 filtered out（5 项消息头、11 项 URI headers）；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增 `rejects_header_with_nul_in_value`，Rust 输入字符串 `"Subject: hello\0world\r\n"`，期望 `Err(String::from("头字段值包含非法控制字符"))`；当前值只做外围空格/HTAB trim，无内容校验，静态预计接受，新反例尚未新增或运行。依据 RFC 3261 §25.1 header-value 和 TEXT-UTF8char 语法，原始 NUL 不允许；本步不涉及折行处理。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 头名空格检查复核：用户新增 Bad Name 反例并以 name.contains(空格) 拒绝名称内空格，目标测试通过，但未验证所有 token 字符。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：15 passed、0 failed、87 filtered out，其中 4 项消息头测试、11 项 URI headers；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步由用户将只检查空格替换为完整 SIP token 字符白名单：ASCII 字母数字加 `-.!%*_+`、反引号、单引号、波浪号；保留名称非空检查，错误沿用头字段名包含非法字符。补 `rejects_header_with_invalid_name_character`，输入 `Bad@Name: hello\r\n`，期望相同错误；现实现静态预计接受，新用例尚未新增或运行。token 中百分号为合法字面字符，不使用 URI 转义或 URI header 校验。规则已查 RFC 3261 §25.1。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 头字段空值反例复核：用户删除非空 filter，将名称非空检查移到 trim 后，新增空行与纯 HTAB 名称两个错误断言；源码复核两处修复正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：14 passed、0 failed、87 filtered out，其中 3 项消息头测试、11 项 URI headers；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增 `rejects_header_with_space_in_name`：输入 `Bad Name: hello\r\n`，期望 `Err(String::from("头字段名包含非法字符"))`；当前仅名称非空检查，静态预计接受内部空格，新测试未新增或运行。RFC 3261 §25.1 定义 header-name 为 token，不能含空格；后续需用正确 token 字符表而非 URI header 字符表。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 头字段错误分支复核：用户修改头字段错误消息并添加空内容/空名称分支。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak header` 退出 0：12 passed、0 failed、87 filtered out，其中只有 1 项消息头正例，其余 11 项为 URI headers 测试；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。静态复核发现两处未修好：strip_suffix 后仍有非空 filter，导致后续 line.is_empty 分支不可达，纯 CRLF 返回缺 CRLF 错误；name.is_empty 位于 trim 前，纯空格/HTAB 名称 trim 后仍会被构造为空名。下一步用户删除该 filter，将名称非空检查移至 trim 后，并添加 rejects_header_with_empty_line 与 rejects_header_with_whitespace_only_name 两个反例，分别输入 CRLF 和纯空白名称加冒号。此二反例尚未新增或运行。代理只读复核及更新工作记录，未改源码或测试；完整工程检查未运行，AC 未完成。

2026-11-09 头字段错误文案评析：用户完成 parse_header 并询问错误消息。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_header` 退出 0：1 passed、0 failed、98 filtered out；仅执行 Via 正例。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。评析：缺 CRLF 文案本身明确，但 `.filter(|v| !v.is_empty())` 将纯 CRLF 的空字段也映射成缺 CRLF 错误，建议拆分成独立空字段检查；缺冒号时的 `header 头错误` 应具体为头字段缺少冒号分隔符。后续名称 trim 后需拒绝空名称；该校验尚未实现、尚未运行对应反例。建议消息统一采用对象+具体原因，不必此时引入错误枚举。用户目前只授权分析讲解，代理未改源码或测试；完整工程检查未运行，AC 未完成。

2026-11-09 单行消息头拆分讲解：用户实现 parse_header，match Some 返回 (&str,&str)、None 提前 return Err，末尾多用 ?。运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_header` 退出 101，E0277：元组不支持 ?，没有测试执行。静态检查还需将拆分的 &str 去掉外围空格/HTAB 并转为 Header 要求的 String；CRLF 错误文案仍为请求行，应改为头字段。当前源码未找到 parse_header 对应测试；指导用户删除 match 后的 ? 并完成构造，然后补 parse_header_with_via。基本拆分不代表头名语法、内嵌换行或折行等边界已完成。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 Header 定义复核：用户已定义 Header，包含 name/value 两个 String，派生 Debug、PartialEq、Eq，静态检查符合当前设计。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；本轮仅新增数据结构，未运行单元测试（0 项执行），未运行完整工程检查。下一步由用户实现单行解析 `parse_header(input: &str) -> Result<Header, String>`：输入 `Via: SIP/2.0/UDP client.example.com:5060\r\n`，移除 CRLF、按第一个冒号拆分名称和值、去除字段外围空格/HTAB，保留值内端口冒号；测试名 `parse_header_with_via`，完整断言 name=Via 和 value=SIP/2.0/UDP client.example.com:5060。后续再补非法头名、折行、紧凑形式和重复头等；目前完整 AC 未完成。代理未改源码或测试。

2026-11-09 password 转义正例与消息头下一步：用户新增 `parse_sip_uri_with_percent_encoded_password`，完整断言合法密码转义按原文保存。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`：85 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 1，测试模块多了一个空行。用户可运行 `RUSTUP_AUTO_INSTALL=0 cargo fmt --all` 格式化。按照 AC 尚未覆盖的 SIP 消息头，下一步由用户在 StartLine 附近定义 Header 结构体，包含 name/value 两个 String，derive Debug、PartialEq、Eq；后续用有序 Vec 保留重复头和未知头。定义本身尚未新增、测试；URI 仍有未覆盖边界，完整 AC 尚未完成。本轮未改源码或测试，完整工程检查未运行。

2026-11-09 password 字符校验复核：用户采用 match/Option 拆分 user/password，对 Some(password) 使用正确字符表并保留 userinfo 全局百分号格式校验和原文。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`：84 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户添加 `parse_sip_uri_with_percent_encoded_password`：输入 `sip:bob:secret%5Eword@example.com`，完整断言 userinfo 为 `Some("bob:secret%5Eword")` 和其余字段，验证密码合法转义及原文保留；静态预计通过，测试尚未新增或运行。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 map_or 解析 userinfo 评析：用户改为 map_or 默认返回 `(userinfo, String::new().as_str())`，两分支均为 `(&str,&str)`，解决此前类型不匹配；但是临时 String 借用在后续使用时可能触发生命周期错误，建议默认直接用空字符串字面量 `""`。此方式将缺失 password 与空 password 映射为相同的临时校验值，原始 userinfo 仍能区分两者；若只校验字符且空 password 合法，此方式可用。我之前推荐 match/Option 是为了明确区分两种状态，未指出 map_or 的可行替代，这是讲解不完整。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`：83 passed、1 failed、13 filtered out，password 尚未使用且非法 password 返回 Ok，编译有 unused variable 警告；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 1，提示 map_or 调用需格式化。下一步用户使用字面量并检查 password 字符集；本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 user 与 password 拆分讲解：用户新增 `rejects_sip_uri_with_invalid_password_character` 并尝试 `userinfo.split_once(...).map_or(userinfo, |(user,pwd)| (user,pwd))`。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak rejects_sip_uri_with_invalid_password_character` 退出 101，两个 E0308：map_or 默认分支为 &str 而闭包分支为 (&str,&str)，类型不一致；没有测试执行。指导以 match 返回统一的 `(&str, Option<&str>)`，区分无 password 与空 password，并在 Some 分支用 password 字符表检查，保留现有百分号校验和 userinfo 原文。参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)。建议实现尚未验证，本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 user 字符校验复核：用户将 userinfo 按第一个冒号取得 user，检查非空并采用正确的 user 字符表，保留现有百分号检查及 userinfo 原文。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`：83 passed、0 failed、0 ignored、13 filtered out，原分号正例与非法 user 反例通过；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步用户新增 `rejects_sip_uri_with_invalid_password_character`，输入 `sip:bob:secret^word@example.com`，期望 `Err(String::from("URI password 包含非法字符"))`（拟定文案）；目前 password 未进行字符校验，静态预计返回 Ok，测试尚未新增或运行。参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)，password 可为空且字符表不同于 user。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 user 校验复用问题：用户用 `validate_uri_header_chars(userinfo).map_err(...)`，顺序正确；单独运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak rejects_sip_uri_with_invalid_user_character` 1 passed、95 filtered out。运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`：82 passed、1 failed、13 filtered out；已有合法 `bob;tag?x` 因 header 字符表不含分号失败。RFC 3261 的 user 允许 `&=+$,;?/`，且不允许 header 字符表的 `[]:`；userinfo 可有 `:password`。下一步用户保留现有百分号校验，拆分 userinfo 中可选 password，非空校验 user，用独立字符表 `-_.!~*'()&=+$,;?/%` 检查 user，保留原始 userinfo；password 校验待下一步。参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)。本轮未改源码或测试，完整工程检查未运行，AC 未完成。

2026-11-09 URI header 名称反例及 user 下一步：用户新增 `rejects_sip_uri_with_invalid_header_name_character`，断言非法名称正确返回 header 字符错误。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`：82 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步由用户新增 `rejects_sip_uri_with_invalid_user_character`：输入 `sip:bo^b@example.com`，期望 `Err(String::from("URI user 包含非法字符"))`，文案为建议；RFC 3261 的 user 语法不允许未转义的 ^。目前 userinfo 只检查非空和百分号转义，静态预计返回 Ok；新测试未新增或运行。参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)。后续实现需保留 user 中合法的分隔符和转义，区分可选 password 的语法。本轮未改源码或测试，完整工程检查未运行，AC 尚未完成。

2026-11-09 URI header 问号正例：用户新增 `parse_sip_uri_with_question_marks_in_header`，完整断言解析结果，名称和值中的 `?` 按原文保存。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`：81 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步由用户补名称含非法原始字符测试 `rejects_sip_uri_with_invalid_header_name_character`：输入 `sip:bob@example.com?sub^ject=hello`，期望 `Err(String::from("URI header 包含非法字符"))`；覆盖名称的调用路径，静态预计通过，测试尚未新增或运行。本轮未改源码或测试，完整工程检查未运行，AC 尚未完成。

2026-11-09 URI header 字符检查复核：用户新增 `validate_uri_header_chars`，先检查百分号转义，再使用 header 字符集校验，名称非空检查后分别应用于名称和值；原外层百分号检查保留。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`：80 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步由用户新增 `parse_sip_uri_with_question_marks_in_header`，输入 `sip:bob@example.com?sub?ject=hello?world`，完整断言 URI 字段及 header 原文 `(sub?ject, hello?world)`，覆盖 header 名和值允许问号；静态预计通过，尚未执行该新测试。参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)。本轮未改源码或测试，完整工程检查未运行，AC 尚未完成。

2026-11-09 URI header 校验与编译错误：用户新增非法 header 值测试并尝试 `validate_uri_param_chars(value)?.map_err(...)`。运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak rejects_sip_uri_with_invalid_header_value_character` 退出 101：编译报 E0599，`?` 后是 `()`，无法调用 `map_err`；0 项测试执行。代码中还写了 `URL header`，与测试期望的 `URI header` 不同。讲解 `map_err` 应在 `?` 前调用，但参数和 URI header 字符集合不一致：header 允许原始 `?`，参数允许 `&`；建议保留外层转义检查，新增 header 字符检查并检查拆出的 name/value，再由用户运行 URI 测试。参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)。本轮未改源码或测试，完整检查未运行。

2026-11-05 参数名转义正例复核与 URI header 下一步：用户新增 `parse_sip_uri_with_percent_encoded_parameter_names`，输入 `sip:bob@example.com;x%5Emode=blue;x%2Fflag`，完整比较六字段；带值和无值参数名称按编码原文及顺序保存，断言正确。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：79 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步转到 URI 内 `?` 后 headers 的字符边界，只由用户新增 `rejects_sip_uri_with_invalid_header_value_character`：输入 `sip:bob@example.com?subject=hello^world`，完整比较 `Err(String::from("URI header 包含非法字符"))`，错误文案为项目拟定。RFC 3261 的 hvalue 仅允许 hnv-unreserved、unreserved、escaped，未转义的 ^ 不在允许集合，参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)。当前 URI headers 分支检查百分号格式、名称非空和等号，但不检查字符集合，静态预计返回 Ok；新测试预计失败，尚未新增或运行。后续在拆出名称和值后校验正确的 URI header 字符集合，保留空值及合法百分号转义；URI 内 headers 与完整 SIP 消息头是不同工作，名称和其他边界待后续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-05 无值参数名反例复核与转义正例下一步：用户在原 `rejects_sip_uri_with_invalid_extension_parameter_name` 内保留带值 `x^mode=blue` 断言，并新增无值 `sip:bob@example.com;x^mode` 同错误断言，使用 Rust 变量遮蔽；两条断言分别验证 Some/None 的参数名称字符检查，写法正确。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：78 passed、0 failed、0 ignored、13 filtered out；新断言位于原测试函数内，因此仍为 78 个测试函数；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `parse_sip_uri_with_percent_encoded_parameter_names`：输入 `sip:bob@example.com;x%5Emode=blue;x%2Fflag`，完整比较六字段，其中 uri_params 按顺序为 `(x%5Emode, Some(blue))` 和 `(x%2Fflag, None)`，名称按原文保留。其余字段 scheme=sip、userinfo=Some(bob)、host=example.com、port=None、uri_headers 为空；该单个样例覆盖带值和无值的合法名称转义，参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 的 pname 和 escaped 规则。静态预计当前实现通过，新测试尚未新增或运行，不将其扩展为 URI 完整验收。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-05 参数字符重复检查整理复核：用户已删除普通扩展值分支中 `validate_uri_param_chars(value)?` 后面的重复字符检查，保留 helper 调用；带值名称和无值名称的 helper 调用也保留。静态复核认可，百分号校验仍由 helper 负责。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：78 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户在现有 `rejects_sip_uri_with_invalid_extension_parameter_name` 里保留 `sip:bob@example.com;x^mode=blue` 原断言，再新增 `sip:bob@example.com;x^mode` 的完整 `Err(String::from("URI 参数包含非法字符"))` 断言；无等号时验证 None 分支的 item 名称也调用字符检查。静态预计当前实现通过，新断言尚未新增或运行；不必新建测试函数，保持此前变量遮蔽风格或自行选择测试形式。合法转义名称等其余 URI 边界仍待后续。

本轮仅更新 CURRENT、任务卡及本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-05 参数字符校验复核与重复块整理：用户提取 `validate_uri_param_chars`：先验证百分号格式，再以正确的 ASCII 字母/数字和 `-_.!~*'()[]/:&+$%` 集合检查字符，返回 `URI 参数包含非法字符`，成功返回 Ok。带值参数名、无值参数名和普通扩展值均调用 helper；此前 `x^mode=blue` 红测已通过，先前扩展值正例及转义错误用例也通过。普通扩展值 else 在调用 helper(value)? 后仍保留完全相同的 `if !value.bytes().all(...)` 检查，同一不可变值已由 helper 检查并在出错时提前返回，因此该块重复且不会再触发错误；并非所有 URI 验收已完成。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：78 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户删除普通扩展值 else 中 `validate_uri_param_chars(value)?;` 后面重复的整个 `if !value.bytes().all(...) { return Err(...) }` 块，保留 helper 调用，随后重新运行 URI 回归。该整理未由代理实施或验证；无值参数名字符边界仍需后续测试。

本轮仅更新 CURRENT、任务卡及本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-05 参数名非法字符红测与复用位置讲解：用户询问参数名检查要写在哪里。已新增 `rejects_sip_uri_with_invalid_extension_parameter_name`，输入 `sip:bob@example.com;x^mode=blue`，完整比较 `Err(String::from("URI 参数包含非法字符"))`。当前有值参数名只做转义格式校验，允许原始 ^。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak rejects_sip_uri_with_invalid_extension_parameter_name` 退出 101：0 passed、1 failed、90 filtered out；实际返回 Ok 且保留 x^mode=blue，符合尚未实现名称字符校验时的红测预期。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，本轮未运行完整 URI 测试组。

会话指导用户提取纯检查函数 `fn validate_uri_param_chars(text: &str) -> Result<(), String>`：先调用 `validate_percent_escapes(text)?`，再检查 ASCII 字母/数字或 `b"-_.!~*'()[]/:&+$%"`，非法返回现有 `URI 参数包含非法字符`，末尾 Ok(())；已有调用处先处理非空。带值 Some((name,value)) 分支把名称非空检查之后的 `validate_percent_escapes(name)?` 替换为该 helper；无值 None 分支 item 即名称，非空检查之后的 `validate_percent_escapes(item)?` 也替换；普通扩展值 else 分支将现有百分号/字符重复检查替换为 helper(value)，其他 transport/user/method、ttl、maddr 专用值分支保留。这样同时覆盖两条参数名路径并保持非法转义原错误、合法转义和单引号；名称/值不被解码。此方案尚未由用户实现或运行；之后运行 URI 回归并逐步补无值名称及合法转义正例。

本轮仅更新 CURRENT、任务卡及本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 扩展值转义正例复核与参数名下一步：用户新增 `parse_sip_uri_with_percent_encoded_extension_parameter_value`，输入 `sip:bob@example.com;x-mode=blue%5Ered`，完整比较六字段并保留参数值编码原文，静态复核认可。合法百分号转义经过扩展值字符检查后仍被接受，已实际验证。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：77 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `rejects_sip_uri_with_invalid_extension_parameter_name`：输入 `sip:bob@example.com;x^mode=blue`，完整比较 `Err(String::from("URI 参数包含非法字符"))`，沿用现有扩展值错误文案。RFC 3261 定义 URI 扩展参数名 pname 为 1*paramchar，原始 ^ 不在允许集合，参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)。当前带值参数名只调用百分号格式校验，静态预计保存 x^mode 并返回 Ok，新测试预计失败，尚未新增或运行。后续应复用现有 paramchar 规则，保留合法转义与合法字符，考虑无值参数名的同一规则和专用参数解析的分支差异，不在此步代理代写。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 扩展值单引号正例复核与转义下一步：用户将允许字符集合改为 `b"-_.!~*'()[]/:&+$%"`，补上合法单引号；新增 `parse_sip_uri_with_apostrophe_in_extension_parameter`，输入 `sip:bob@example.com;x-note=it's`，完整比较六字段并保留参数值原文。静态复核认可字符表、转义检查顺序及断言。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：76 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `parse_sip_uri_with_percent_encoded_extension_parameter_value`：输入 `sip:bob@example.com;x-mode=blue%5Ered`，完整比较六字段，scheme sip、userinfo Some(bob)、host example.com、port None、uri_params 一个 `(x-mode, Some(blue%5Ered))`、uri_headers 空。该正例用于实际验证扩展值经过百分号格式检查和字符集合检查后保留编码原文；此前未找到针对通用扩展值的合法转义正例。静态预计当前实现通过，新测试尚未新增或运行；参数名和其他 URI 规则后续继续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 扩展值字符校验复核与单引号遗漏：用户在普通扩展参数有值分支保留 `validate_percent_escapes(value)?`，随后用 ASCII 字母/数字及字节字符集合检查非法值，返回与现有断言一致的 `URI 参数包含非法字符`；`blue^red` 错误路径已验证。静态复核发现实际集合为 `-_.!~*()[]/:&+$%`，漏了 RFC 3261 unreserved mark 中允许的 ASCII 单引号 `'`；例如 `sip:bob@example.com;x-note=it's` 当前静态预计错误拒绝，该具体样例尚未运行。参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 的 mark 与 paramchar。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：75 passed、0 failed、0 ignored、13 filtered out，原非法字符红测通过；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告；当前测试未覆盖合法单引号，不将绿灯等同完整字符集正确。

下一小步由用户在字节字符串中加入单引号，例如 `b"-_.!~*'()[]/:&+$%"`，并新增正例 `parse_sip_uri_with_apostrophe_in_extension_parameter`：输入 `sip:bob@example.com;x-note=it's`，完整比较六字段，uri_params 保存 `(x-note, Some(it's))` 原文。此修正和新测试尚未实施或运行；保留百分号格式检查，参数名、合法转义及其他 URI 规则后续继续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 扩展参数非法值红测与实现讲解：用户询问扩展参数值字符检查如何写。已新增实际测试名 `rejects_sip_uri_with_invalid_extension_paramter_value`，输入 `sip:bob@example.com;x-mode=blue^red`，实际预期为 `Err(String::from("URI 参数包含非法字符"))`，与此前建议文案略有差异；讲解沿用用户实际文案，不要求改名或更改断言。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak rejects_sip_uri_with_invalid_extension_paramter_value` 退出 101：0 passed、1 failed、87 filtered out，实际返回 Ok 且保存 x-mode=blue^red，与预期 Err 不同；这是当前尚未实现字符检查的预期红测。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0。完整 URI 测试组本轮未运行。

会话指导用户只在通用有值 else 分支的 `validate_percent_escapes(value)?` 后检查：value.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.!~*'()[]/:&+$%".contains(&b))，若为 false 返回 `String::from("URI 参数包含非法字符")`。允许集合来自 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 的 paramchar、unreserved 与 escaped；先执行现有转义检查保证允许的 `%` 必须配两位十六进制。既有非空检查在前，校验后仍保存原文，不解码；合法的 `%5E` 不应因该字符集合被误拒。该建议只覆盖普通扩展参数有值分支，参数名、无值参数名及已识别专用参数的其他语法仍待后续。示意未由代理写入或执行，下一步由用户添加后重跑 URI 测试；本轮不声称新测试通过。

本轮仅更新 CURRENT、任务卡及本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 扩展参数正例复核与非法值下一步：用户新增 `parse_sip_uri_with_extension_parameters`，输入 `sip:bob@example.com;x-mode=blue;x-flag`，完整比较六字段；有值参数与无值参数分别保存 Some(blue)、None，顺序与原名保留，断言正确。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：74 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `rejects_sip_uri_with_invalid_extension_parameter_value`：输入 `sip:bob@example.com;x-mode=blue^red`，完整比较 `Err(String::from("URI 参数值包含非法字符"))`，错误文案为项目拟定。RFC 3261 定义 other-param 的值由 paramchar 构成，原始 `^` 不在允许集合，应使用转义表示；参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 中 paramchar 及 unreserved 定义。当前通用有值分支仅检查百分号转义，静态预计保存并返回 Ok，新测试预计失败，尚未新增或运行。后续应按 URI 参数字符集合校验，并保留正确百分号转义及合法扩展字符；名称、无值扩展参数等其他边界待逐步验证，不降低原有合法输入支持。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 method 正例复核与扩展参数下一步：用户新增 `parse_sip_uri_with_method_parameter`，输入 `sip:bob@example.com;method=REGISTER`，完整比较六字段，method 参数值以原文 REGISTER 保存；静态复核认可断言。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：73 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `parse_sip_uri_with_extension_parameters`：输入 `sip:bob@example.com;x-mode=blue;x-flag`，完整比较六字段：scheme sip、userinfo Some(bob)、host example.com、port None、uri_params 依次为 `(x-mode, Some(blue))` 和 `(x-flag, None)`、uri_headers 空。用一条样例验证有值与无值合法扩展参数及原顺序，参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 的 other-param/pname/pvalue。静态预计当前实现通过，新测试尚未新增或运行；这只验证保存，不表示实现扩展参数业务含义，也不代表完整 URI 语法或本 AC 验收。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 无值 method 参数复核与正常样例下一步：用户新增实际函数名 `rejects_sip_uri_with_valueless_method_paramter`，输入 `sip:bob@example.com;method`，完整比较 `Err(String::from("method 参数必须包含值"))`；并在无等号 None 分支使用 item.eq_ignore_ascii_case("method") 返回该错误，其他无值参数保留。静态复核认可分支及大小写处理；已有 lr 正例本轮通过。测试函数名中的 paramter 为用户实际拼写，不要求为此修改。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：72 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `parse_sip_uri_with_method_parameter`：输入 `sip:bob@example.com;method=REGISTER`，完整比较六字段：scheme sip、userinfo Some(bob)、host example.com、port None、uri_params 一个 `(method, Some(REGISTER))`、uri_headers 空。method 参数正例有助于验证用户已实现的 token 格式接受和原字符串保留；静态预计当前实现通过，新测试尚未新增或运行。扩展方法与其他 URI 规则仍待后续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 user=phone 参数保存复核与无值 method 下一步：用户新增 `parse_sip_uri_with_user_phone_parameter`，实际输入 `sip:bob@example.com;user=phone`，完整比较六字段，userinfo 保存 bob，uri_params 保存 user=phone，断言已实际验证字段拆分与原文保存。该样例使用 bob 而非此前建议的电话号码，因此不能称其验证了电话号码实例或电话号码语义；如希望样例体现 user=phone 表示电话号码的含义，可以把输入和期望的 bob 都换为 +12125551212，但本次不要求改名或强制修改。参考 [RFC 3261 §19.1.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-19.1.1)。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：71 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `rejects_sip_uri_with_valueless_method_parameter`，输入 `sip:bob@example.com;method`（无等号和值），完整比较 `Err(String::from("method 参数必须包含值"))`，错误文案为项目拟定。[RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 指定 method-param 为 method=及 Method 值。当前无等号 None 分支仅对 ttl、maddr、transport、user 专门拒绝，method 走通用无值参数并保存，静态预计返回 Ok；新断言预计失败，尚未新增或运行。后续应仅对已识别 method 参数大小写无关处理，保留 lr 和其他合法无值扩展参数；含等号空值走 Some 分支既有共享空值检查，正例与其他 URI 规则仍待后续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 无值 user 参数复核与 phone 正例下一步：用户新增 `rejects_sip_uri_with_valueless_user_parameter`，输入 `sip:bob@example.com;user`，完整比较 `Err(String::from("user 参数必须包含值"))`；并在无等号 None 分支使用 item.eq_ignore_ascii_case("user") 返回该错误，其他无值参数保留。静态复核认可分支及大小写处理；既有 lr 正例本轮通过。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：70 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `parse_sip_uri_with_user_phone_parameter`：输入 `sip:+12125551212@example.com;user=phone`，完整比较六字段：scheme sip、userinfo Some(+12125551212)、host example.com、port None、uri_params 一个 `(user, Some(phone))`、uri_headers 空。URI 中的 user 参数说明 user 部分的解释方式，不是 userinfo 字段本身；参考 [RFC 3261 §19.1.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-19.1.1) 和 §19.1.3 样例。静态预计当前实现通过，新测试尚未新增或运行；method 无值及其他 URI 规则仍待后续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 无值 transport 分支复核与无值 user 参数下一步：用户新增 `rejects_sip_uri_with_valueless_transport`，实际输入 `sip:bob@exmple.com;transport`，完整比较 `Err(String::from("transport 参数必须包含值"))`；实际域名与建议例子拼写不同但仍合法，不影响该用例。用户在无等号 None 分支中，以 item.eq_ignore_ascii_case("transport") 返回专用错误，保留 ttl、maddr 专用错误与其他无值参数；静态复核认可，已有 lr 正例本轮通过。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：69 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `rejects_sip_uri_with_valueless_user_parameter`，输入 `sip:bob@example.com;user`（无等号和值），完整比较 `Err(String::from("user 参数必须包含值"))`，错误文案为项目拟定。这里 user 是 URI 参数，不是 @ 前的 userinfo。[RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 指定 user-param 为 user=及 phone、ip 或扩展值。当前无等号 None 分支只对 ttl、maddr、transport 专门拒绝，user 走通用无值参数并保存，静态预计返回 Ok；新断言预计失败，尚未新增或运行。后续应仅对已识别 user 参数大小写无关处理，保留 lr 和其他合法无值扩展参数；含等号空值走 Some 分支既有共享空值检查。method 等其他 URI 规则仍待后续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-11-03 maddr 域名测试复核与无值 transport 下一步：用户新增 `parse_sip_uri_with_hostname_maddr`，输入 `sip:bob@example.com;maddr=proxy.example.net`，完整比较六字段，主 host 保留 example.com，maddr 参数值原文保存 proxy.example.net；静态复核认可该正常样例。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：68 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。maddr 的 hostname、IPv4、IPv6 已有正例，不将其视为完整 URI 或 AC 验收。

下一小步只由用户新增 `rejects_sip_uri_with_valueless_transport`，输入 `sip:bob@example.com;transport`（无等号和值），完整比较 `Err(String::from("transport 参数必须包含值"))`，错误文案为项目拟定。[RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 指定 transport-param 为 transport=及传输值。当前无等号 None 分支只对 ttl 与 maddr 专门拒绝，transport 走通用无值参数并保存，静态预计返回 Ok；新断言预计失败，尚未新增或运行。后续应仅对已识别 transport 做大小写无关处理，保留 lr 和其他合法无值扩展参数；含等号空值走 Some 分支既有共享空值检查。SIPS/传输关系及其他 URI 规则仍待后续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 无值 maddr 分支复核与域名正例下一步：用户新增 `rejects_sip_uri_with_valueless_maddr`，输入 `sip:bob@example.com;maddr`，完整比较 `Err(String::from("maddr 参数必须包含值"))`；并在无等号 None 分支对 item.eq_ignore_ascii_case("maddr") 返回该错误，保留 ttl 独立错误和其他无值参数。静态复核认可大小写无关及位置；既有含 lr 的正例在本轮通过。`maddr=` 仍经 Some 分支共享空值检查，不混同缺等号错误。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：67 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `parse_sip_uri_with_hostname_maddr`，输入 `sip:bob@example.com;maddr=proxy.example.net`，完整比较六字段：scheme sip、userinfo Some(bob)、主 host example.com、port None、uri_params 一个 `(maddr, Some(proxy.example.net))`、uri_headers 空。现有 IPv4 和 IPv6 maddr 正例已有，域名形式缺正例；静态预计当前实现通过，新测试尚未新增或运行。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr IPv6 附端口测试复核与无值下一步：用户新增 `rejects_sip_uri_with_ipv6_maddr_port`，实际输入 `sip:bob@example.com;maddr=[2001:db8::1]:5056`，完整比较 `Err(String::from("maddr host 无效"))`；5056 与此前示例 5060 同样验证方括号后附端口被拒绝，无需修改。该错误路径已实际验证。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：66 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `rejects_sip_uri_with_valueless_maddr`，输入 `sip:bob@example.com;maddr`（无等号和值），完整比较 `Err(String::from("maddr 参数必须包含值"))`，错误文案为项目拟定。[RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 的已识别 maddr 参数形式为 `maddr=host`。当前无等号 None 分支只对 ttl 专门拒绝，maddr 走通用无值参数并保存，静态预计返回 Ok；新测试预计失败，尚未新增或运行。后续只应在 None 分支对 maddr 大小写无关处理，保留合法 lr 和其他无值扩展参数；`maddr=` 的空值走不同路径、已有共享空值检查。完整 URI 语法及其余边界仍待后续。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr 缺括号测试复核与额外端口下一步：用户新增独立测试 `rejects_sip_uri_with_unclosed_ipv6_maddr`，输入 `sip:bob@example.com;maddr=[2001:db8::1`，完整比较 `Err(String::from("maddr host 无效"))`；静态复核认可，已实际验证 maddr 缺右括号的错误路径。上轮磁盘未找到断言的记录为当时状态，此轮已找到新增测试。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：65 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `rejects_sip_uri_with_ipv6_maddr_port`，输入 `sip:bob@example.com;maddr=[2001:db8::1]:5060`，完整比较 `Err(String::from("maddr host 无效"))`，验证 maddr 的 IPv6 闭括号后不能附加端口。[RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 定义 maddr-param 为 host 而非 hostport；URI 的 port 是主 hostport 的另一组件。静态预计现有 strip_suffix 检查能通过该断言，新测试尚未新增或运行；空地址等其他边界后续验证。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr 缺括号测试清单核对：用户回复“通过了”。只读核对磁盘上的 src/main.rs：`rejects_sip_uri_with_invalid_ipv6_maddr` 当前只含 `[not-ipv6]` 一条断言，未发现 `maddr=[2001:db8::1` 缺右括号测试或断言；已有 `rejects_sip_uri_with_ipv6_missing_closing_bracket` 针对主 URI host `sip:bob@[::1`，不覆盖 maddr。可能有未保存的编辑，本轮不推断用户未编写。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 退出 0，列出 64 项，未找到 maddr 缺括号的独立测试；list 不执行测试，本轮实际执行 0 项。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 与 `git diff --check` 退出 0；上轮 64 项 URI 测试通过只是历史结果，不能说本轮 maddr 缺括号反例已验证。

下一步请用户确认对应更改已保存；也可在现有 `rejects_sip_uri_with_invalid_ipv6_maddr` 保留原断言并追加 `parse_sip_uri("sip:bob@example.com;maddr=[2001:db8::1")`，完整比较 `Err(String::from("maddr host 无效"))`。该输入和主 host 缺括号用例走不同分支；静态预计通过，尚未实际运行。

本轮仅更新 CURRENT 与任务卡，未改源码、测试、进度或需求；未运行完整 URI 测试组、Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。完整 URI 与本 AC 验收仍未完成。

2026-10-29 maddr 非法 IPv6 测试复核与缺括号下一步：用户新增 `rejects_sip_uri_with_invalid_ipv6_maddr`，输入 `sip:bob@example.com;maddr=[not-ipv6]`，完整比较 `Err(String::from("maddr host 无效"))`；方括号完整但内容无效，实际验证 Ipv6Addr 解析错误路径，断言正确。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：64 passed、0 failed、0 ignored、13 filtered out；13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。

下一小步只由用户新增 `rejects_sip_uri_with_unclosed_ipv6_maddr`，输入 `sip:bob@example.com;maddr=[2001:db8::1`，完整比较 `Err(String::from("maddr host 无效"))`，验证缺少右方括号时 strip_suffix 的错误路径。静态预计现有实现能通过该断言，新测试尚未新增或运行；IPv6 方括号后缀、空地址等其他边界后续逐项验证。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr IPv6 正例复核与非法地址下一步：用户已将 IPv6 maddr 正例的主 host 期望改为 `example.com`，保留 uri_params 的 `[2001:db8::1]` 原文；maddr 分支对 `[` 开头的值先去除完整首尾方括号，再用 `parse::<std::net::Ipv6Addr>()` 校验，错误映射为 `maddr host 无效`；其他值仍调用 `validate_hostname_or_ipv4` 并映射错误。静态复核认可分支、错误传播和保存原文方式。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：63 passed、0 failed、0 ignored、13 filtered out；包含 IPv6 maddr 正常样例与已有域名/IPv4 等 URI 测试，13 项起始行/CLI 单元测试过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 未报告警告。上轮单测失败已修复，不把正例扩展为全部 IPv6 边界已验证。

下一小步只由用户新增 `rejects_sip_uri_with_invalid_ipv6_maddr`，输入 `sip:bob@example.com;maddr=[not-ipv6]`，完整比较 `Err(String::from("maddr host 无效"))`，验证方括号完整但内部不是合法 IPv6 的错误路径。静态预计当前实现通过该断言，但新测试尚未新增或运行；缺闭括号、多余后缀、空地址等边界后续逐项验证。

本轮仅更新 CURRENT、任务卡与本项备注并 render；代理未改源码或测试，状态和验收字段不变，完整 URI 与其余本 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr IPv6 实现讲解与测试期望修正：用户询问 maddr IPv6 代码如何写；已新增 `parse_sip_uri_with_ipv6_maddr`，输入 `sip:bob@example.com;maddr=[2001:db8::1]`，但期望 `host` 错写为 `[2001:db8::1]`。URI 主 host 是 `example.com`，方括号 IPv6 是 maddr 参数值；应把测试期望 host 改成 String::from("example.com")，保留 uri_params 中的方括号 IPv6 原文。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak parse_sip_uri_with_ipv6_maddr` 退出 101，实际 0 passed、1 failed、75 filtered out：当前结果为 Err("maddr host 无效")，现有期望为 Ok，因尚无 maddr IPv6 分支；测试期望 host 的第二处问题目前尚未进入成功值比较。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 与 `git diff --check` 退出 0。未将以前62项通过作为本轮实际结果，也未运行完整 URI 测试集。

会话讲解 maddr 分支示意：若 value.starts_with('[')，用 strip_prefix('[') 与 strip_suffix(']') 取得内部文字，缺括号或多余后缀返回 `String::from("maddr host 无效")`，再将内部文字 parse::<std::net::Ipv6Addr>() 并用 map_err 映射同一错误；否则保留 `validate_hostname_or_ipv4(value).map_err(...)`。`ok_or_else` 闭包直接返回 String，`?` 向上传播；解析结果只验证，参数保存仍用原始 value。不允许将所有方括号开头值直接当成功，也不支持 maddr 中附端口。建议用户先修正 host 期望并完成该分支，再重跑 URI 测试；IPv6 错误边界及其他规则随后逐项补测，尚未实施或验证。

本轮只更新 CURRENT、任务卡和本项备注并 render；代理未改源码或测试，状态及验收字段保持。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr hostname 复用复核与 IPv6 正例下一步：用户将 maddr 有值分支改为 `validate_hostname_or_ipv4(value).map_err(|_| String::from("maddr host 无效"))?`，删除了原先的 contains/starts_with 特例；域名与 IPv4 复用现有字符和标签校验，原两条非法 hostname 测试及现有 IPv4 maddr 样例通过。当前所有 maddr 值都送进只接受 hostname/IPv4 的函数，该函数拒绝方括号和冒号，静态预计合法 `[2001:db8::1]` 也会被拒绝。尚未新增或运行 maddr IPv6 正例，既有主 host 的 IPv6 测试不能证明 maddr IPv6 受支持。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：62 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 与 `git diff --check` 退出 0，test target 未报告警告。13 项起始行/CLI 单元测试被过滤未运行。

下一小步只由用户新增 `parse_sip_uri_with_ipv6_maddr`：输入 `sip:bob@example.com;maddr=[2001:db8::1]`，完整比较六字段：scheme `sip`、userinfo Some(`bob`)、host `example.com`、port None、uri_params 为一个 `(maddr, Some([2001:db8::1]))`（值原文含方括号）、uri_headers 为空。[RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 规定 maddr 为 host，host 可为 IPv6reference。新测试当前预计失败，尚未新增或运行；之后再指导用户在 maddr 分支判断方括号、校验完整括号边界并解析内部 IPv6，错误映射为 maddr host 无效，同时保留非方括号的共享函数调用。缺闭括号、无效地址和额外尾部均需拒绝，后续逐项验证。

本轮仅更新 CURRENT、任务卡与本项备注并 render，状态及验收字段不变，代理未改源码或测试。完整 URI、消息及本 AC 验收未完成；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 hostname/IPv4 校验函数提取复核：用户将原 `if !host.starts_with('[')` 内部 hostname 字符、标签、IPv4/toplabel 校验提取为 `validate_hostname_or_ipv4(host: &str) -> Result<(), String>`，以原错误文案返回并在成功末尾 `Ok(())`；原位置在非 IPv6 分支调用 `validate_hostname_or_ipv4(host)?`。host 非空与 IPv6 分支保留，代码结构符合前一小步；现有 maddr 分支仍只检查下划线和整串前导连字符。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：62 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 与 `git diff --check` 退出 0，test target 未报告警告。62 个 URI 测试函数通过，13 项起始行/CLI 单元测试被过滤未运行；本轮未新增 maddr IPv6 用例，不把当前结果扩大为完整 URI 验收。

下一小步由用户让 maddr 值做 host 校验：若以 `[` 开头，要求恰有完整方括号包住地址，取内部文本调用 `parse::<std::net::Ipv6Addr>()`；缺闭括号、括号后多余内容或地址无效均返回项目错误 `maddr host 无效`。其余值调用 `validate_hostname_or_ipv4(value).map_err(|_| String::from("maddr host 无效"))?`，替换当前 contains/starts_with 零散判断。公共空值检查已在前面；方括号 IPv6 不能送入 hostname/IPv4 函数，也不能只凭 `[` 就接受。示例期望：`maddr=foo.-bad.example.com` 错误；`maddr=239.255.255.1` 与 `maddr=[2001:db8::1]` 合法；`maddr=[not-ipv6]` 错误。这些新样例仅为实施目标，尚未全部新增或运行。完成实现后再由用户逐项补测试；IPv6 端口或其他尾部不属于 host，应拒绝。无值 maddr 及其余 URI 规则仍待后续。

本轮仅更新 CURRENT、任务卡与本项备注并 render，in_progress 和验收字段不变，代理未改源码或测试。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr 缩写说明：用户询问 maddr 全称。结合 RFC 2543 的历史多播用法，可将 maddr 通常理解为 Multicast Address（多播地址）的缩写；这是名称的常见理解，不声称 RFC 3261 明文定义该展开。已复核 RFC 3261 §19.1.1：SIP URI 中 maddr 指定需要联系的服务器地址，实际值也可为非多播地址。当前不改变原有 hostname/IPv4 校验提取学习下一步。

本轮仅术语说明并同步 CURRENT 与任务卡，Rust 测试执行 0 项，未读取或修改源码、测试或进度；上轮62项URI通过仅是历史结果。未运行 fmt、完整工程检查或网络实验，未提交或推送。

2026-10-29 maddr 前导连字符复核与校验提取安排：用户在现有 `rejects_sip_uri_with_invalid_maddr_host` 保留原下划线断言，新增 `sip:bob@example.com;maddr=-bad.example.com` 的相同完整错误断言，并将 maddr 分支扩为 `value.contains('_') || value.starts_with('-')`；这两个样例实际被拒绝。该检查仍是局部实现，例如 `foo.-bad.example.com` 的中间标签以连字符开头但整个值不以连字符开头，静态预计仍被接受，尚未实际测试；其他 hostname 规则与 maddr IPv6 也未完整校验。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：62 passed、0 failed、0 ignored、13 filtered out。仍是 62 个 URI 测试函数，其中 maddr 错误测试现有两条断言；13 项起始行/CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 和 `git diff --check` 退出 0，test target 无警告。

下一小步只由用户提取现有 hostname/IPv4 校验：定义 `fn validate_hostname_or_ipv4(host: &str) -> Result<(), String>`，把 `if !host.starts_with('[')` 内部现有字符、标签、IPv4/toplabel 逻辑搬入，成功尾部 `Ok(())`；在原条件内改为 `validate_hostname_or_ipv4(host)?`。host 非空检查与 IPv6 方括号/地址校验暂留原处，保留原错误文案和现有 maddr 分支，先运行现有 URI 回归；后续再学习如何将共享校验用于 maddr 和处理其 IPv6。这个助手仅接收 hostname/IPv4，不可直接用于方括号 IPv6。提取尚未由用户实现或验证，代理不代写源码或测试。

本轮仅更新 CURRENT、任务卡及 AC-01-01 备注并 render 生成视图，状态及验收字段不变；完整 URI 和其余 AC 验收未完成。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr 下划线校验评审与下一步：用户新增 `rejects_sip_uri_with_invalid_maddr_host`，实际输入 `sip:bob@exmaple.com;transport=udp;maddr=bad_host.example.com`，完整比较 `Err(String::from("maddr host 无效"))`；`exmaple.com` 虽与建议拼写不同，仍是合法 hostname，不影响该测试。新增 maddr 有值分支以 `value.contains('_')` 拒绝下划线，该输入已正确处理，但这只是部分 host 校验；静态推导 `maddr=-bad.example.com` 等其他非法 hostname 及非法 IPv6 仍会被接受，maddr 专用分支还绕过了通用转义检查，整体尚未完成。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：62 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 与 `git diff --check` 均退出 0，本轮 test target 无警告。11 项起始行及 2 项 CLI 单元测试被过滤未运行。

下一小步由用户在现有 `rejects_sip_uri_with_invalid_maddr_host` 中保留原断言并新增一组输入 `sip:bob@example.com;maddr=-bad.example.com`，同样完整比较 `Err(String::from("maddr host 无效"))`。RFC 3261 规定 maddr 为 host，hostname 标签开头不能是连字符；标签内部的连字符可合法使用，例如 `edge-1.example.com`。参考 [RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1)。当前仅检查下划线，预计新断言失败；新断言尚未创建或运行。后续讨论提取并复用现有 URI host 校验，同时处理 IPv6 边界，不应逐个禁止每种字符或禁止所有连字符。

仅同步 CURRENT、任务卡与本项备注并 render 更新生成视图，in_progress 与验收字段不变，代理未改源码或测试。完整 URI、消息及序列化验收仍未完成；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络实验，未提交或推送。

2026-10-29 maddr 概念讲解：用户询问 maddr 是什么。已核对 RFC 3261 §19.1.1/25.1 与 RFC 3263 §4：maddr 是 SIP URI 参数，指定请求联系目标的 host，存在时覆盖从 URI host 得到的地址；port 和 transport 应用于该目标。语法为 maddr=host，值可为域名、IPv4 或 IPv6，不限于多播；示例 sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=15 中，bob@example.com 表示逻辑资源，UDP 为传输方式，239.255.255.1 为多播目标，ttl 控制多播 IP 包的传播范围。URI 的 ttl 仅适用于 UDP 和多播 maddr；RFC 不鼓励以 maddr 实现松散源路由。

当前 parse_sip_uri 中 maddr 尚无专用分支，只经过普通百分号检查并保存参数原文；尚未验证 maddr 的 host 语法或 ttl 与 UDP/多播的跨参数条件，也不执行网络发送。此前建议的 `rejects_sip_uri_with_invalid_maddr_host`（maddr=bad_host.example.com）尚未创建，仍作为用户理解后的下一小步；本轮 Rust 测试执行 0 项，上轮61项URI通过仅作历史结果。仅同步 CURRENT 与任务卡，不改源码、测试或进度；未运行 fmt、完整工程检查或网络实验，未提交或推送。

2026-10-29 TTL 合法下界复核与 maddr 下一步：用户新增实际函数名 `parse_sip_uri_with_minimum_ttl_parameter`，输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=0`，完整比较六字段，参数顺序保持、TTL 原文保存为字符串 0，断言正确；minimum 与此前建议 zero 表达同一合法下界，无需改名。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：61 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 与 `git diff --check` 均退出 0，test target 未报告警告。本轮 11 项起始行和 2 项 CLI 单元测试被过滤，未重复运行。

下一小步转到 maddr 的 host 格式，只建议用户新增 `rejects_sip_uri_with_invalid_maddr_host`：输入 `sip:bob@example.com;maddr=bad_host.example.com`，完整比较 `Err(String::from("maddr host 无效"))`，错误文案为项目拟定。[RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 的 maddr-param="maddr=" host 与 hostname 标签规则要求该值为合法 host，主机名不能含下划线。当前 maddr 走通用百分号转义检查后保存，没有 host 检查，静态预计返回 Ok，因此新断言预计失败；尚未创建或实际运行。先由用户补这一条，再讨论复用已有 host 校验，不代写解析器或测试。maddr 的必值、IPv6、其他专用参数及 ttl 使用条件等规则仍待后续，不将目前结果视为完整 URI 或本 AC 验收。

本轮仅同步 CURRENT、任务卡及 AC-01-01 备注，并由 render 更新生成视图；in_progress 与验收字段保持，代理未改源码或测试。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。

2026-10-29 进度核对与 TTL 下界下一步：用户询问当前进度与下一步。已核对任务卡、源码和状态真源：AC-01-01 为 in_progress，当前在五步学习顺序的第 2 步 SIP URI。起始行已有基础解析；URI 已具备 scheme、userinfo、host、port、uri_params、uri_headers 六字段及部分格式/边界检查。URI 内的 headers 不等于完整 SIP 消息头；消息头、重复 Via/Contact、原始 body 和序列化仍待后续。AC-01-02/03/04 均为 not_started。

上次建议删除的 TTL Some 分支末尾重复 value.is_empty 块已由用户删除，前方共享空值检查与 None 分支无值 ttl 检查仍保留。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak` 退出 0：73 passed、0 failed、0 ignored、0 filtered out，包含 60 项 URI、11 项起始行和 2 项 CLI 单元测试；test target 未报告警告。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 与 `git diff --check` 均退出 0。

下一小步只由用户新增 `parse_sip_uri_with_zero_ttl_parameter`：输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=0`，参考现有 255/003 正例完整比较六字段，并保留 TTL 原字符串 0 及参数顺序。[RFC 3261 §25.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-25.1) 规定 ttl=1*3DIGIT、0..255；该用例补合法下界，尚未新增或实际运行，依现有逻辑预计通过。其他专用参数与 URI 规则仍须逐步补齐，不扩大为完整 URI 或本 AC 验收。

本轮仅同步 CURRENT、任务卡和 AC-01-01 备注，并由 render 更新生成视图；状态及验收字段不变，代理未改源码或测试。未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。

2026-10-29 重复空值判断整理复核通过：用户已删除 TTL 有值分支末尾不可达的重复 value.is_empty 块，保留前方共享空值检查与 None 分支无值 ttl 拒绝。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：60 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。

用户要求下一步，建议补合法下限正例 `parse_sip_uri_with_minimum_ttl_parameter`：输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=0`，完整比较六字段、三个参数按原顺序保存，其中 ttl 为 Some("0")；可参照已有255正例并仅改输入与期望中的 ttl 值。已核对 RFC 3261 的 ttl=1*3DIGIT、数值0至255，0是合法下限；此步仅验证解析和原文保留，不涉及实际发送。当前静态预计正例直接通过，新测试尚未创建或实际运行。完成后仍需其他 URI 专用参数及完整消息等学习/验收。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13项起始行/CLI单元测试及完整工程检查未运行，未提交或推送。

2026-10-29 无值 ttl 分支实现评审通过：用户新增无等号 ttl 的完整错误断言，并在 split_once('=') 的 None 分支中，名称非空/转义校验后以 item.eq_ignore_ascii_case("ttl") 返回专用必值错误，其他无值项继续保存。本步位置、大小写处理与返回逻辑正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：60 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告，既有 lr 正例仍通过。

唯一局部整理建议：删除 Some 分支 TTL 校验末尾 src/main.rs:222–224 新加的 value.is_empty 块；前面的共享空值检查已在198–200返回，后面的空值分支永远不会执行。;ttl 走None并返回“ttl 参数必须包含值”，;ttl=走Some并在共享检查返回“URI 参数值不能为空”，两条路径不混用。静态及独立只读复核认可，建议尚未代写或验证，不要求改名或无关重构。下一步由用户仅删除这三行并复跑；本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13项起始行/CLI单元测试及完整工程检查未运行，完整ttl/URI验收仍未完成，未提交或推送。

2026-10-29 ttl 前导零原文保留复核通过：用户新增 `parse_sip_uri_with_leading_zeros_in_ttl_parameter`，输入带 UDP、多播maddr、ttl=003，完整比较六字段，参数顺序保持且 TTL 保存为字符串003。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：59 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。003 合法性与原文保留已由真实正例验证，不只依据静态推导。

下一小步建议用户新增 `rejects_sip_uri_with_valueless_ttl_parameter`，输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl`（TTL 后没有等号和值），完整比较 Err(String::from("ttl 参数必须包含值"))，文案为项目拟定。RFC 3261 的 ttl-param="ttl=" ttl 要求带值；当前无等号 None 分支会直接保存 (ttl,None)，静态及独立只读复核预计返回 Ok，新断言预计失败，尚未创建或运行。后续须在 None 分支仅对无值 ttl 大小写无关识别并拒绝，保持 lr 与未知合法无值参数；现有 Some 分支 value.is_empty 无法覆盖缺等号路径。合法0及其余ttl/URI规则仍待后续，不扩大验收。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13项起始行/CLI单元测试及完整工程检查未运行，未提交或推送。

2026-10-29 ttl 位数检查修正复核通过与003讲解：用户已删除零字符 any，TTL 分支在 ASCII 数字检查后以 len()>3 返回长度错误，再做 u8 范围检查。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：58 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。四位0000实际被长度检查拒绝，合法255与范围外256仍分别成功/拒绝，前轮E0308及格式问题已解决。

用户询问003是否合法；已重新核对 RFC 3261 ttl=1*3DIGIT、0..255，003为三位十进制、数值3，语法合法；本模型验证后仍保存原文003，不能改成3。下一步建议新增 `parse_sip_uri_with_leading_zeros_in_ttl_parameter`，输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=003`，完整比较六字段，可参考255正例，仅把输入与期望的 ttl 字符串改为003，其他参数顺序和字段保持一致。当前静态及独立只读复核预计直接通过，尚未创建或实际运行。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13项起始行/CLI单元测试及完整工程检查未运行，完整ttl/URI验收仍未完成，未提交或推送。

2026-10-29 ttl 位数检查讲解：用户新增四位0000的完整错误断言，已写 value.len()>3，但又加入 value.bytes().any(|b| b"0".contains("0")) 且 if 块未填返回。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101，实际报 E0308：字节集合 contains 需要 &u8，却传入 &str，测试执行 0 项；fmt 退出 1，要求折叠空 if 块，git diff --check 退出 0。未沿用上轮57项通过为本轮结果。

仅会话建议删掉整个零字符 any 条件，保留 if value.len()>3 并返回 Err(String::from("ttl 参数必须是 1 到 3 位数字"))，放在 ASCII 数字检查之后、u8 范围转换之前。共享空值检查已确保至少1位，ASCII检查通过后每个字节就是一位数字，所以只需补最大长度3；1..3位指字符数量，0和000都合法，0000仅因四位而非法，不应禁止零字符。静态及独立只读复核认可，建议尚未代写或执行；下一步用户完成这一处局部修改并格式化、复跑。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；完整工程检查及完整ttl/URI验收未完成，未提交或推送。

2026-10-29 ttl 合法上限正例复核通过：用户新增 `parse_sip_uri_with_maximum_ttl_parameter`，输入带 transport=udp、maddr=239.255.255.1、ttl=255，完整比较六字段，三个参数按输入顺序与原字符串保存。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：57 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。255 成功与256拒绝均受现有测试保护，位数规则仍待补。

下一步只建议用户新增 `rejects_sip_uri_with_ttl_parameter_longer_than_three_digits`，输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=0000`，完整比较 Err(String::from("ttl 参数必须是 1 到 3 位数字"))，文案为项目拟定。RFC 3261 ttl=1*3DIGIT；0000 全为数字且转换为0不超出u8，四位长度独立违反语法，能隔离位数限制而非数值范围。当前只有非空、数字格式与 u8 范围检查，静态及独立只读复核预计返回 Ok，新断言预计失败，尚未创建或实际运行。合法0及允许位数内的前导零原文保留后续覆盖，不提前代写实现。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整ttl/URI验收仍未完成，未提交或推送。

2026-10-29 ttl 范围转换与测试分隔符修正复核通过：用户已将共享 value.is_empty 检查提前到名称转义检查后、参数分类前；TTL 数字格式后以 parse::<u8>().is_err() 返回范围错误；range 测试使用分号进入真实参数分支。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：56 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。E0308 及格式问题已解决，ttl=256 实际被范围检查拒绝，原空值错误路径保持。

下一步建议用户新增 `parse_sip_uri_with_maximum_ttl_parameter`，输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=255`，完整比较六字段：scheme=sip、userinfo=Some(bob)、host=example.com、port=None、uri_headers=[]，uri_params 按原顺序保存 (transport,Some(udp))、(maddr,Some(239.255.255.1))、(ttl,Some(255)) 字符串。255 是合法上限，此正例保护边界成功与参数原文/顺序保留；静态及独立只读复核预计直接通过，尚未创建或实际运行。合法0与1..3位等后续逐步覆盖，不认定完整ttl/URI已完成。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-29 ttl 范围转换类型与测试分隔符诊断：用户新增 range 测试，但输入误写为 sip:bob@example.com?transport=udp;maddr=239.255.255.1;ttl=256；实现使用 if let Some(ttl)=value.parse::<u8>() 后返回范围错误。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101，实际报 E0308（parse 返回 Result<u8, ParseIntError>，却用 Option 的 Some 匹配），测试执行 0 项；fmt 退出 1，要求去掉 src/main.rs:212 的 if 后多余空格；git diff --check 退出 0。未把类型错误或格式失败记为测试运行结果。

只建议配套三处局部修正：在 TTL ASCII 数字格式检查之后用 value.parse::<u8>().is_err() 判断转换失败并返回既定范围错误，u8 的上限即255，转换成功无需取数值或 unwrap；将现有共享 value.is_empty() 检查整块移至 validate_percent_escapes(name)? 后、参数类型分类前，保持空值原错误，避免空串被误归为范围错误；range 测试首分隔符 ? 改为 ;，完整输入为 sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=256，确保实际进入参数路径。问号表示 URI headers，当前错例即使修好类型也不能验证 ttl 参数范围（静态判断，未实际运行）。静态及独立只读复核认可，建议尚未代写或执行；位数1..3仍待后续。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；完整工程检查与完整 URI 验收未完成，未提交或推送。

2026-10-29 ttl 大小写数字检查复核通过：用户已将分类改为 eq_ignore_ascii_case("ttl")，现有 `rejects_sip_uri_with_non_numeric_ttl_parameter` 保留小写 ttl=abc 并追加大写 TTL=abc，两组均完整比较同一格式错误。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：55 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。新增断言未增加测试函数数，大小写绕过已被覆盖；ttl 范围/位数仍未检查。

下一步仅建议用户新增 `rejects_sip_uri_with_out_of_range_ttl_parameter`，输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=256`，完整比较 Err(String::from("ttl 参数超出 0..255 范围"))，文案为本项目拟定。256 是三位十进制，格式正确但超出 RFC 3261 的 0..255（含255）数值范围；现有 ASCII 数字检查仍会接受，静态及独立只读复核预计新断言失败，尚未创建或运行。先补此范围红测，合法边界与位数后续逐步覆盖，参数继续按原模型保存原文。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-29 ttl 含义讲解与小写数字检查复核：用户询问 ttl 含义并要求下一步。已核对 RFC 3261 §19.1.1/25.1 与 RFC 1812 §5.3.1：此处 Time To Live 设置 UDP 多播包的 IP TTL，用于限制传播范围，路由转发时至少递减一、耗尽丢弃；SIP URI 的 ttl 配合 UDP 与多播 maddr 使用，文法 1..3 位十进制、数值 0..255。当前为纯解析校验，不执行网络转发。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：55 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，前轮语法及行尾空白问题已消失。

用户已补数字检查，但参数分类仍为 name.eq("ttl")，未按前轮建议改为忽略大小写；大写 TTL=abc 会进入普通 else 被接受。下一步先在现有 `rejects_sip_uri_with_non_numeric_ttl_parameter` 追加 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;TTL=abc`，同样完整比较 Err(String::from("ttl 参数必须是十进制数字"))，保留小写用例，仍计一个函数。静态及独立只读复核预计新增断言失败，尚未创建或运行；随后分类使用 eq_ignore_ascii_case("ttl")。先补此遗漏，位数/范围与合法值后续逐步覆盖。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-29 ttl 十进制格式讲解：用户新增 `rejects_sip_uri_with_non_numeric_ttl_parameter`，输入带 transport=udp、maddr=239.255.255.1、ttl=abc，完整比较用户拟定的 Err("ttl 参数必须是十进制数字")；实现已在参数分类中增加 name.eq("ttl") 分支，但停在未完成的 if。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101，实际为 expected expression/found } 语法错误，测试执行 0 项；fmt 检查退出 1，因同一语法错误无法解析；git diff --check 退出 2，src/main.rs:209 行尾空白，均未记为通过。

仅会话给局部建议：沿用现有端口检查的 value.bytes().all(|b| b.is_ascii_digit())，在 ttl 分类内以 !all 拒绝任一非 ASCII 0..9 字节，返回与当前测试一致的“ttl 参数必须是十进制数字”；名称分类改用 eq_ignore_ascii_case("ttl")，与 URI 参数名大小写规则一致。保留之后共享 value.is_empty() 检查，因为 all 对空字符串为 true，但空值仍需由现有分支拒绝。该步骤仅验证数字格式，15 可接受格式、abc/0x10 不可；位数 1..3 和数值 0..255 后续补，不能外推 ttl 完整校验。静态与独立只读复核认可，建议尚未代写或实际验证；下一步由用户补完整分支、格式化并复跑 URI 回归。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；完整工程检查及完整 URI 验收未完成，未提交或推送。

2026-10-29 非顶层标签数字开头正例复核通过：用户新增 `parse_sip_uri_with_hostname_label_starting_with_digit`，输入 sip:bob@1example.com，完整比较六字段，非顶层标签 1example 与顶层 com 的作用范围正确。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：54 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。仅认可当前已练习 host 边界，不扩大为完整 URI 或本 AC 验收。

下一小步回到 URI 专用参数，仅建议新增 `rejects_sip_uri_with_non_numeric_ttl_parameter`，输入 `sip:bob@example.com;transport=udp;maddr=239.255.255.1;ttl=abc`，完整比较 Err(String::from("ttl 必须是十进制数字"))，文案为本项目拟定。本轮 rfc-editor 抓取失败，改用 IETF 官方 RFC 3261 核对 ttl-param→ttl、ttl=1*3DIGIT（0..255）以及仅用于 UDP 多播 maddr 的约束；用例显式给 UDP/多播地址以隔离 abc 数字格式错误。当前 ttl 仍走普通 else 的百分号检查，静态及独立只读复核预计返回 Ok，新断言预计失败，尚未创建或运行。长度、范围、合法值及 maddr 等其他规则后续逐步覆盖，不提前代写实现。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-29 顶层标签校验与迭代器修正复核通过：用户保留外层非 IPv6 检查、内部合法 IPv4 跳过顶层标签的结构，host_for_labels 已恢复为字符串切片，分别 split/rsplit 创建迭代器，starts_with 闭包明确 char。该分层方案与可选合并方案均可表达本步规则，不要求为统一形式重写。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：53 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。example.1com 拒绝、合法 IPv4/IPv6、合法尾点及空标签用例同时通过，E0282/E0382 已解决。

下一步建议用户新增 `parse_sip_uri_with_hostname_label_starting_with_digit`，输入 `sip:bob@1example.com`，完整比较六字段：scheme=sip、userinfo=Some(bob)、host=1example.com、port=None、uri_params=[]、uri_headers=[]。依据已核对 RFC 3261 §25，非顶层标签可以数字开头，最后的 com 仍从字母开始；该正例保护顶层字母规则不被扩大到全部标签。静态及独立只读复核预计直接通过，尚未创建或运行。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整 hostname/URI 验收仍未完成，未提交或推送。

2026-10-29 host 三类分支讲解：用户询问外层 !host.starts_with('[') 能否因新增 IPv4 判断而替换。只读核对说明前者表示非 IPv6，仍包含 IPv4 与 hostname；非 IPv4 仍包含 IPv6 与 hostname，二者不能单独互换。仅会话建议集中分类：let is_ipv6=host.starts_with('[')、let is_ipv4=host.parse::<Ipv4Addr>().is_ok()，仅在 !is_ipv6 && !is_ipv4 内放原有 hostname 字符、标签及顶层标签检查，去掉内部重复 IPv4 判断。IPv6 内容已在前面由 Ipv6Addr 验证，方括号用于识别既有类别，不重复解析；未代写该结构调整。

用户已将 starts_with 闭包改成显式 char，E0282 已消失；本轮 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101，实际报 E0382：host_for_labels.last() 移动迭代器后又调用 any，测试执行 0 项。下一步仍需按前轮建议保留 host_for_labels 为字符串切片，分别创建迭代器，三类分类整理也可由用户一并完成；尚未验证任何建议。仅同步 CURRENT 与任务卡，不改源码、测试或进度；本轮未重跑 fmt 或完整工程检查，未提交或推送。

2026-10-29 顶层标签初版编译诊断：用户已新增 `rejects_sip_uri_with_hostname_top_label_starting_with_digit`，输入 sip:bob@example.1com，完整比较 hostname 错误；实现将 host_for_labels 绑定成 split 迭代器，调用 last 后又 any，并对所有非 IPv6 host 检查顶层字母。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101，当前实际编译错误为 E0282（starts_with 闭包参数 p 类型无法推断），测试执行 0 项；fmt 与 git diff --check 均退出 0，不沿用上轮 52 项通过作为本轮结果。

已对照本机固定 Rust 1.96.0 标准库源码及独立只读复核：starts_with 的谓词闭包接收 char，可显式写 |ch: char|；Iterator::last(self) 消耗迭代器，之后再 any 存在静态所有权问题（当前被 E0282 挡住，未宣称已实际触发该后续错误）；合法 IPv4 不能套 hostname 顶层字母规则。仅会话给局部建议：host_for_labels 恢复为 strip_suffix('.') 后的 &str，不接 split、无需 mut；以其 split 创建独立迭代器保留空标签/首尾连字符检查，再用原始 host.parse::<Ipv4Addr>().is_ok() 判断 IPv4，仅对非 IPv4 用 host_for_labels.rsplit('.').next().unwrap_or("") 取最后标签并以类型明确的 starts_with 检查 ASCII 字母。使用原 host 识别 IPv4，校验视图仅剥一个尾点，最终仍保留原 host。建议尚未代写或执行，下一步由用户完成局部修改复跑；本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度，完整工程检查及完整 hostname/URI 仍未完成，未提交或推送。

2026-10-28 IPv4 host 正例复核通过并继续顶层标签学习：用户已新增 `parse_sip_uri_with_ipv4_host`，输入 `sip:bob@192.0.2.1`，完整比较六字段，host 原文保留。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：52 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。该结果仅保护合法 IPv4 输入，不代表地址范围或完整 host 校验已完成。

用户要求继续，下一小步只建议新增 `rejects_sip_uri_with_hostname_top_label_starting_with_digit`，输入 `sip:bob@example.1com`，完整比较 Err(String::from("hostname 无效"))。已核对的 RFC 3261 §25 中 toplabel 从 ALPHA 开始；1com 虽包含字母但以数字开头，因此该例不满足 hostname 语法。当前仅检查字符集合、空标签与首尾连字符，静态预计仍返回 Ok，新断言预计失败，尚未创建或运行。规则只针对剥去可选尾点后的 hostname 最后一段，不要求每个标签以字母开头（如 1example.com 可合法），也不能套到 IPv4 分支；先由用户补测试再讨论最小实现。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-28 单尾点/双尾点修正复核通过：用户已将空标签测试第三组改为 sip:bob@example.com..，移除新增首尾点拒绝条件，保留字符集合/逐标签检查；原合法单尾点完整正例保持。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：51 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。空标签函数含中间空段、开头空段、双尾点三组断言，仍只计一项测试；单尾点与双尾点的期望冲突已消除。

下一步先建议用户新增 `parse_sip_uri_with_ipv4_host`，输入 `sip:bob@192.0.2.1`，完整比较六字段：scheme=sip、userinfo=Some(bob)、host=192.0.2.1、port=None、空 uri_params/uri_headers。当前静态及独立只读复核预计直接通过，尚未创建或运行。已重新核对 RFC 3261 §25 的 host=hostname/IPv4address/IPv6reference，toplabel 首字母规则只约束 hostname；先保护合法 IPv4，避免后续补 hostname 顶层标签规则时误拒数字地址，不将该正例外推为 IPv4 范围或完整 host 校验已完成。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-28 空标签新增断言的单尾点/双尾点混淆诊断：用户在原空标签测试追加开头点输入，但第三组误写为单尾点 sip:bob@example.com. 并期望 Err，而原合法尾点正例对同一输入期望 Ok；同时新增 has_invalid_with_staring_or_ending=host.starts_with('.') || host.ends_with('.') 并加入拒绝条件，导致合法单尾点被误拒。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101：50 passed、1 failed、0 ignored、13 filtered out，唯一失败为合法尾点正例实际收到 hostname 错误；fmt 与 git diff --check 均退出 0，test target 未报告警告。

只建议两处局部修正：第三组输入按原目标改成双尾点 sip:bob@example.com..，仍完整比较原错误；删除新增首尾点 flag 及 if 中对应引用，保留 !has_only_valid_chars || has_invalid_label，合法单尾点正例及期望不变。既有 strip_suffix 仅剥一次，再检查空标签已能拒绝开头点和双尾点，同时允许单尾点；同一单尾点输入不能同时要求 Ok 和 Err。建议尚未代写或执行，下一步由用户修正两处并复跑，不再追加新任务。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整 hostname/URI 仍未完成，未提交或推送。

2026-10-28 空标签校验实现复核通过：用户已按非 IPv6 路径采用 host.strip_suffix('.').unwrap_or(host) 的校验视图，在一次 split/any 中检查标签为空或首尾连字符，与 has_only_valid_chars 组合返回既定错误；最终 host.to_string 继续保存原文。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：51 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。example..com 拒绝与 example.com. 原文正例同时通过，前轮迭代器编译错误已解决。

下一步建议用户在现有 `rejects_sip_uri_with_empty_hostname_label` 再追加 `sip:bob@.example.com`、`sip:bob@example.com..` 两组输入，各完整比较 Err(String::from("hostname 无效"))，保留原中间空标签用例，合计一项测试三组输入。分别覆盖开头空标签与仅剥一个尾点后仍为空标签的边界，当前静态及独立只读复核预计新增断言均通过；追加断言尚未创建或实际运行，URI 测试函数数量不因此增加。保留合法单尾点正例，完整 hostname/toplabel 等规则仍待后续。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-28 空标签校验思路与迭代器编译错误讲解：用户新增 `rejects_sip_uri_with_empty_hostname_label`，输入 sip:bob@example..com，完整比较 hostname 错误；实现停在 let parts=host.split('.')、let has_label_middle_with_hyphen=parts.into_iter()，随后再 parts.any。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101，编译报 E0382（into_iter 消耗并移动 parts 后又使用）及 E0596（绑定 parts 非 mut 却调用 any），另有新增变量未使用警告；测试执行 0 项，不沿用上一轮 50 项通过作为本轮结果。fmt 与 git diff --check 均退出 0。

仅在会话给局部建议：保留非 IPv6 外层与 has_only_valid_chars，将从 let parts 到最终 hostname 错误 if 的部分替换为两步：let host_for_labels=host.strip_suffix('.').unwrap_or(host)，只为校验忽略一个可选尾点；let has_invalid_label=host_for_labels.split('.').any(label 为空或以连字符开头/结尾)，再以 !has_only_valid_chars || has_invalid_label 返回原 hostname 错误。split 已生成迭代器，into_iter 不会检查空标签；直接链式一次检查，无需 clone、收集或重复消费。最终仍 host.to_string 保存原文，example.com. 可保留合法性，example..com / example.com.. 会留下空标签。静态与独立只读复核认可，建议尚未写入或运行；下一步由用户完成此局部修改再复跑。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度，完整工程检查及完整 hostname/URI 规则未完成，未提交或推送。

2026-10-28 合法 hostname 尾点原文保留复核通过：用户新增 `parse_sip_uri_with_trailing_dot_in_hostname`，输入 `sip:bob@example.com.`，完整比较六字段，host 保持 example.com.，其余为正常 sip/bob、无端口、空参数与空 headers。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：50 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。此正例保护后续空标签检查的合法尾点边界。

下一步建议用户新增 `rejects_sip_uri_with_empty_hostname_label`，输入 `sip:bob@example..com`，完整比较 Err(String::from("hostname 无效"))。连续点之间存在空标签，不满足 RFC 3261 §25 的 domainlabel；当前只做字符集合与标签首尾连字符检查，静态预计接受该输入，新断言预计失败，尚未创建或运行。后续校验可在仅去掉一个可选尾点的借用视图上逐标签检查非空，仍保存原始 host；不能 trim 任意多个尾点，也不能直接把合法尾点拆出的最后空段当错误。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；完整 hostname/URI 规则、其余 13 项单元测试及完整工程检查未完成，未提交或推送。

2026-10-28 标签首尾连字符拒绝复核通过：用户采用 has_only_valid_chars、has_label_starting_or_ending_with_hyphen 与闭包 label，逐标签以 starts_with('-') || ends_with('-') 表达两类非法位置，并新增 ending_with_hyphen 用例，完整比较 a.example-.com 的 hostname 错误。静态与只读复核认可本步逻辑，长变量名含义准确可保留。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：49 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。

下一步建议先保护合法尾点：用户新增 `parse_sip_uri_with_trailing_dot_in_hostname`，输入 `sip:bob@example.com.`，完整比较 SipUri(scheme=sip, userinfo=Some(bob), host=example.com., port=None, uri_params=[], uri_headers=[])，保留 host 原文。RFC 3261 §25 的 hostname 允许一个可选尾点；当前静态预计此正例直接通过，尚未创建或运行。随后再补空标签拒绝时不能误拒该合法边界；本步通过不代表完整 hostname/URI 已完成。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试与完整工程检查未运行，未提交或推送。

2026-10-28 标签前导连字符实现与命名评审通过：用户在非 IPv6 路径中保留 all 字符校验，并增加 host.split('.').any(|text| text.starts_with('-'))，以 !valid_chars || invalid_chars 返回原 hostname 错误。该 any 的谓词描述非法位置，本步逻辑正确，完整断言得到验证。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：48 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。

用户询问更好写法与变量名；静态及独立只读复核建议保留现有 all/any 结构，仅在会话建议 valid_chars→has_only_valid_chars、invalid_chars→has_label_starting_with_hyphen、闭包 text→label，分别清楚表达布尔字符集合结果、存在前导连字符标签及点分标签；当前 invalid_chars 检查的是位置，并非非法字符集合。建议尚未代写或运行。下一小步建议 `rejects_sip_uri_with_hostname_label_ending_with_hyphen`，输入 `sip:bob@a.example-.com`，完整比较 Err(String::from("hostname 无效"))；按已核对 RFC 3261 §25 的 domainlabel 尾部 alphanum 规则，此标签非法，当前静态仍会返回 Ok，预计新断言失败，尚未创建或运行。完整标签/URI 规则仍待后续。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-28 字符集合与标签位置规则讲解：用户已新增实际名 `rejects_sip_uri_with_hostname_label_startig_with_hyphen`，输入 `sip:bob@a.-example.com`，完整比较 hostname 错误，并询问为何点与连字符已经检查仍失败。单独执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak tests::rejects_sip_uri_with_hostname_label_startig_with_hyphen -- --exact` 退出 101：0 passed、1 failed、0 ignored、60 filtered out；实际返回 Ok(host=a.-example.com)，预期为 Err("hostname 无效")，已直接复现。

仅讲解当前谓词 b.is_ascii_alphanumeric() || b"-.".contains(&b) 将连字符与点列为允许字符，因此本输入所有字节都通过 all，valid_chars=true，不进入拒绝分支；字符集合通过仍需单独验证标签位置。对照 my-example（连字符在标签中间）与 -example（标签开头）说明规则；按点分成 a、-example、com 后，后续应逐标签检查起始字符，而不能只检查整个 host 开头。此处按前轮已核对的 RFC 3261 §25 解释，不新建要求。下一步仍由用户在现有非 IPv6 分支内补标签前导连字符校验，保持允许字符检查，不改测试期望。本轮仅同步 CURRENT/任务卡，不改源码、测试或进度；其余 60 项单元测试、fmt 与完整工程检查未运行，上一轮 47 项 URI 通过只作历史结果，未提交或推送。

2026-10-28 hostname 字符集合回归恢复通过：用户已删除旧注释块，在 host 非空检查后使用两层 if：非方括号 host 以 all(ASCII 字母数字或 b"-.") 得到 valid_chars，再取反返回原 hostname 错误；IPv6 保持前面的专用验证。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：47 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告。普通合法 URI、下划线/加号拒绝和 IPv6 相关用例均恢复通过，本步认可条件及位置，不代表完整 hostname/URI 已完成。

下一步只建议用户新增 `rejects_sip_uri_with_hostname_label_starting_with_hyphen`，输入 `sip:bob@a.-example.com`，完整比较 Err(String::from("hostname 无效"))。已核对 RFC 3261 §25：点分开的 domainlabel 不能以连字符开头；此例定位中间标签 -example，当前字符集合检查仍会接受，静态预计新断言失败，尚未创建或运行。随后按点拆分普通 hostname、逐标签验证；合法尾点、空标签、标签结尾及 toplabel 等规则仍待后续，不能把这一项当作完整标签语法。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试和完整工程检查未运行，未提交或推送。

2026-10-28 hostname 条件拆层讲解：用户已注释原 hostport 判断，将新检查放在拆出 host 且非空之后，允许集合改为 b"-."，位置与集合均符合本步目标。但新块仍用 any(合法字符) 后返回 Err(String::from("host "))，且没有跳过已验证 IPv6。实际 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101：27 passed、20 failed、0 ignored、13 filtered out；普通合法 URI 和 IPv6 实际收到 host 空尾文案错误，非法 hostname 测试则因错误文案与预期不同失败。fmt 与 git diff --check 均退出 0，test target 未报告警告。

本轮不重复合并长条件，而仅建议用户替换新 hostname 块：外层 if !host.starts_with('[') 仅检查普通 host；内部 let valid_chars = host.bytes().all(ASCII 字母数字或连字符/点)，再 if !valid_chars 返回既定 Err(String::from("hostname 无效"))。用 example.com→true→不报错、exa_mple.com/含加号→false→报错说明先算合法性，再决定拒绝，避免将 any(合法)直接当非法。保持空 host、百分号和 IPv6 专用验证；建议仅在会话提供，未代改源码或实际验证。下一步由用户完成这处修改复跑 URI 回归；不新增测试或后续任务。仅同步 CURRENT 与任务卡，进度不变；13 项起始行/CLI 单元测试、完整工程检查与完整 hostname 标签规则仍未完成，未提交或推送。

2026-10-28 学习续接复盘：用户询问昨天停在哪里。已重新核对任务卡、当前源码与正式进度：第 01 章 AC-01-01 仍为 in_progress，学习顺序停在第 2 步 SIP URI；起始行基础解析已有代码，URI 的 scheme/userinfo/host/port、IPv6、参数、URI headers 与百分号等路径已逐步练习，完整 URI 及后续消息头、Via/Contact、body/序列化仍未完成。第 01 章 AC-01-02/03/04 均为 not_started，不以测试数量推算进度百分比。

昨天最后未解决的仍是 hostname 字符判断：当前 src/main.rs:188–193 的 any 正向条件仍含 is_ascii_alphanumeric（字节串现为 b"_"，与昨天记录 b"_+" 略有不同，但根因不变），正常域名遇 e 就返回 hostname 错误。今日实测 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101：14 passed、33 failed、0 ignored、13 filtered out。今天先由用户移除该块，在拆出 host 且确认非空后、写回前，对非 IPv6 host 以 !all(ASCII 字母数字或连字符/点) 拒绝非法字符，保留百分号与 IPv6 专用检查；然后复跑同一 URI 命令，不先增加新任务。本轮仅复盘及同步 CURRENT/任务卡，未改源码、测试或进度；未重跑 fmt、其余 13 项单元测试或完整工程检查，未提交或推送。

2026-10-27 hostname 允许字符判断回归诊断：用户在原 invalid_hostname_character 测试补了 exa+mple.com，断言正确；实现却改成 hostport.bytes().any(|b| b.is_ascii_alphanumeric() || b"_+".contains(&b)) 后返回 hostname 错误。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101：14 passed、33 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak tests::parse_sip_uri_with_userinfo -- --exact` 退出 101：0 passed、1 failed、59 filtered out，正常 example.com 实际返回 hostname 错误。fmt 与 git diff --check 均退出 0，未把测试失败记为通过。

实际断言、源码与独立只读复核确认：any 的正向条件把任意字母/数字也当作拒绝理由，example.com 在 e 处即触发；_+ 还是禁止字符而非允许集合，且检查对象仍为混合 hostport。只提供局部建议、不代改：删除当前 hostname 检查块，保留既有百分号检查；在拆出 host 且确认非空后、写回前，对非方括号 IPv6 的 host 使用 !bytes().all(ASCII 字母数字或连字符/点)，任一字符不允许才返回原错误。IPv6 已由前面专用分支验证，避免把其括号/冒号当作域名字符。建议尚未写入或运行，不能声称恢复通过；下一步由用户完成这处修改并复跑 URI 回归。确定性现有用例已直接定位原因，未增加日志、二分或新协议测试；标签结构与完整 URI 仍待后续。本轮仅同步 CURRENT 与任务卡，进度不变；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 hostname 下划线拒绝与写法评审：用户新增 `rejects_sip_uri_with_invalid_hostname_character`，输入 `sip:bob@exa_mple.com`，完整比较 hostname 错误；当前 hostport.bytes().any 配合单元素字节串可正确完成此目标。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：47 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，test target 未报告警告，上一轮多余括号已去除。

静态及独立只读复核建议：单字符判断可直接写 hostport.contains('_')；但不能把这一条规则视为完整 hostname 校验。按 RFC 3261 §25，普通 hostname 的字符来自 ASCII 字母、数字、连字符与点，当前 `sip:bob@exa+mple.com` 会漏检（静态推导，未创建或运行新用例）。下一步由用户在现有 invalid_hostname_character 测试补该输入，同样完整比较 Err(String::from("hostname 无效"))，先暴露漏检；随后改为允许字符集合检查，放在拆出 host 且确认非空后、写回字段前，跳过已有 Ipv6Addr 验证的方括号 IPv6。不能直接在混合 hostport 上使用该集合，否则端口冒号和 IPv6 会被误拒；标签起止/空标签等完整规则仍待后续。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 host 百分号拒绝复核通过：用户新增 `rejects_sip_uri_with_percent_encoded_host`，输入 `sip:bob@%65xample.com`，完整比较 host 百分号错误；在 hostport 与参数分离后检查 hostport.contains('%')，对本步目标有效。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：46 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。编译有一条 unused_parens 警告，条件外括号可去掉，未把警告记为测试失败。

下一步建议用户新增 `rejects_sip_uri_with_invalid_hostname_character`，输入 `sip:bob@exa_mple.com`，完整比较 Err(String::from("hostname 无效"))。已核对 RFC 3261 §25 的 hostname/domainlabel/toplabel，下划线不属于 SIP hostname 允许字符；当前普通 host 尚无此验证，静态推导返回 Ok，新断言预计失败，尚未创建或运行。字符检查应针对拆出的普通 hostname，保留 IPv6 的独立验证，不在整条 input 上检查；此步也不代表标签起止等完整域名规则完成。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 合法 URI header 编码值原文保留复核通过：用户新增 `parse_sip_uri_with_percent_encoded_header_value`，输入 `sip:bob@example.com?subject=hello%20world`，完整比较正常字段与 uri_headers=[(subject, hello%20world)]。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：45 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `rejects_sip_uri_with_percent_encoded_host`，输入 `sip:bob@%65xample.com`，完整比较 Err(String::from("host 不能包含百分号转义"))。已核对 RFC 3261 §19.1.2 明确 host 不允许百分号转义；当前普通 host 只有非空等基础检查，静态推导会接受 %65xample.com，新断言预计失败，尚未创建或运行。此步单独拒绝 host 中的编码，不将其解码后当作域名，也不代表完整 hostname 语法已完成。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 URI headers 非法转义复核通过：用户新增实际名 `reject_sip_uri_with_invalid_percent_escape_in_headers`，两组输入分别在名称和值中含 %2G，完整比较百分号错误；Some(headers_text) 在按 & 拆分前调用 helper，空 header 值规则保持。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：44 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

当前只读复核认为整段 headers 的百分号形状预检可保留：hname 与 hvalue 共享 %HH 规则，原始 & 和 = 分隔符不可能充当十六进制位，不存在参数 token 值的同类例外；此结论只针对转义格式，不代表完整字符验证。下一步建议用户新增 `parse_sip_uri_with_percent_encoded_header_value`，输入 `sip:bob@example.com?subject=hello%20world`，完整比较正常 scheme/userinfo/host、port=None、空参数及 uri_headers=[(subject, hello%20world)]，保护合法转义原文保留。按当前实现静态推导应直接通过，尚未创建或运行。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整 URI 仍未完成，未提交或推送。

2026-10-27 有值/无值参数名转义复核通过：用户新增 `rejects_sip_uri_with_invalid_percent_escape_in_parameter_name`，实际输入 `sip:bob@example;fo%2G=bar` 与 `sip:bob@example;fo%2G`，均完整比较百分号错误；无等号分支已在非空检查后调用 validate_percent_escapes(item)?。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：43 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `rejects_sip_uri_with_invalid_percent_escape_in_headers`，同一测试两组输入为 `sip:bob@example.com?sub%2Gject=hello` 与 `sip:bob@example.com?subject=%2G`，各完整比较 Err(String::from("URI 百分号转义无效"))，分别覆盖 URI header 名与值。当前 headers 分支尚未调用 helper，静态推导两组会返回 Ok，预计新测试失败；尚未创建或运行。随后在拆出名/值、确认名称非空后分别验证，空 header 值继续允许。此处仅指 URI 的 ?headers。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整 URI 仍未完成，未提交或推送。

2026-10-27 参数名称/值调用修正复核通过：用户已在名称非空检查后恢复 validate_percent_escapes(name)?，普通参数 else 正确调用 validate_percent_escapes(value)?；token 值字符检查保持。静态与独立只读复核认可。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：42 passed、0 failed、0 ignored、13 filtered out，foo=%2G 拒绝与 transport=x%y 保留均通过；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `rejects_sip_uri_with_invalid_percent_escape_in_parameter_name`，同一测试两组输入分别为 `sip:bob@example.com;fo%2G=bar` 与 `sip:bob@example.com;fo%2G`，每组完整比较 Err(String::from("URI 百分号转义无效"))。静态推导第一组由现有 name 检查拒绝，第二组走无等号分支，仅检查 item 非空而漏掉转义校验，预计新测试失败；尚未创建或运行。先覆盖该邻近分支，再继续 headers 等其他组件。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整 URI 仍未完成，未提交或推送。

2026-10-27 参数值分支实参错误诊断：用户已添加 transport/user/method 的真实 token 字符检查，但 else 分支误写 validate_percent_escapes(name)?，仍未检查 value；原分类前名称检查也被移走。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101：41 passed、1 failed、0 ignored、13 filtered out；唯一失败仍为 foo=%2G 返回 Ok，transport 正例通过。fmt 与 git diff --check 均退出 0。

实际断言与独立只读复核确认最小两处建议：将 else 的实参 name 改为 value；在 name 非空检查后、参数种类分类前恢复 validate_percent_escapes(name)?。foo=%2G 中 name 是 foo，非法转义位于 value=%2G，分类条件及 token 字符集合本轮已正确；value 空值检查可后续整理，不是此次根因。只给局部修改说明，不代改源码或测试；建议尚未实际验证。下一步由用户完成这两处后复跑 URI 回归。仅同步 CURRENT 与任务卡，进度不变；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 参数名分类条件讲解：用户询问为何比较 transport/user/method。仅解释该条件用 name 选择 value 的校验规则：RFC 的 other-transport、other-user、extension-method 都使用 token，可在当前词法步骤共享字符集合检查；普通 foo 参数走 other-param 的 pvalue/escaped。三个名字来自 SIP 语法，非任意挑选；eq_ignore_ascii_case 用于参数名大小写无关比较，正向 || 表示任意一个名称匹配即进入 token 分支。此处 user 指 URI 参数 ;user=phone，不是 @ 前的 userinfo。

当前源码仍未补 value 分类校验，上一轮已复现的 foo=%2G 失败未宣称修复；本轮 Rust 测试执行 0 项，不沿用上轮结果为本轮通过。maddr、ttl 等其他专门规则仍待后续，不把参数划分为仅有两个完整类别。下一步由用户在 value 非空检查后补上真实的 token 校验/普通值百分号校验，再运行 URI 回归。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度，未运行完整工程检查，未提交或推送。

2026-10-27 参数值漏校验回归诊断：用户新增 transport=x%y 的完整正常期望，并移除了整段参数扫描；当前仅在有等号参数分支校验 name，value 只检查非空。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101：41 passed、1 failed、0 ignored、13 filtered out。新 transport 正例已通过，唯一失败为旧 foo=%2G 拒绝用例，实际返回 Ok 并保存 (foo, Some(%2G))；fmt 与 git diff --check 均退出 0。

根因已由实际断言及独立只读复核确认：检查的是名称 foo，而非法转义位于值 %2G。会话仅给局部补法：保留 name 的检查，在 value 非空检查后、返回元组前按 ASCII 大小写无关的参数名分类；transport/user/method 的值实际验证 RFC token 字符集合（非空已由前置检查保证），其余值在当前步骤调用 validate_percent_escapes(value)?，保留原文。不是以空成功分支跳过 token 校验；maddr、ttl 等专门语法及无值参数/header 路径仍属未完成范围，不据此声明完整参数合法性。建议片段未写入或实际运行，下一步由用户补此分支再跑 URI 回归。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 校验函数提取复核：用户已正确提取 validate_percent_escapes，成功返回 Ok(())，并在 userinfo 非空检查后调用。参数部分却仍在 split(';') 之前对整段 params_text 调用，未按参数种类分类；transport 正例尚未新增，原误拒边界仍存在。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：41 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，仅认可现有用例与已完成的提取，不记为检查范围问题已解决。

独立只读复核确认：src/main.rs:185 的 validate_percent_escapes(text)? 仍会静态拒绝 `sip:bob@example.com;transport=x%y`。下一步继续由用户新增 `parse_sip_uri_with_percent_in_transport_token`，完整比较正常 scheme/userinfo/host/port、参数 [(transport, Some(x%y))] 与空 headers，先使漏测边界可见；该用例未创建或实际运行，静态预计失败。之后再将参数校验移到逐项拆出名和值、识别参数种类之后，保留 foo=%2G 的拒绝；不分散到新的 headers 用例。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 百分号校验放置讲解：用户询问应放在哪里，当前源码仍为入口全 input 扫描，transport 正例尚未新增。本轮只给位置与接口建议：将原扫描提取为 parse_sip_uri 旁的 `validate_percent_escapes(text: &str) -> Result<(), String>`，扫描 text、成功末尾返回 Ok(())；入口保留空白/控制字符检查。调用放在组件已拆好、保存原文之前：userinfo 非空检查后；URI headers 每项拆出名/值后；普通 other-param 分类分支的名称及可选值校验处。不能扫描整段 params_text，也不能对 transport/user/method 的 token 值、maddr 的 host、ttl 的数字语法统一套用 %HH。

独立只读复核确认调用位置与边界。完成移动时要同时接好 userinfo 和普通参数的校验；仅接 userinfo 会让已有 foo=%2G 拒绝测试重新失败，不将该中间状态记为完成。下一步仍由用户补 transport 正例、提取函数并按参数种类调用，不用专用参数直接 Ok(()) 作占位。未代改源码、测试或进度；本轮 Rust 测试执行 0 项，未重跑 fmt 或完整工程检查，上轮 41 项通过仅保留为历史结果。本轮只同步 CURRENT 与任务卡，未提交或推送。

2026-10-27 参数值非法转义测试通过但全局扫描需调整：用户新增实际名 `rejects_sip_uri_with_invalid_precent_escape_in_parameter_value`，输入 `sip:bob@example.com;foo=%2G`，完整比较百分号错误；同时将原 userinfo 扫描移至整个 input，而非按组件复用。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：41 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0，不把现有测试通过等同于此次扫描范围完全正确。

RFC 3261 §19.1.2 要求依组件 BNF 决定原始字符；§25.1 的 token 含字面 %，transport-param→other-transport→token。依据该语法及两份独立只读复核，`sip:bob@example.com;transport=x%y` 可在解析模型中保留为扩展 token，当前入口全局扫描却会提前返回百分号错误，静态判断属于本次扩大检查范围导致的误拒；尚未创建或实际运行此例。下一步建议用户新增 `parse_sip_uri_with_percent_in_transport_token`，完整比较正常字段与 uri_params=[(transport, Some(x%y))]，先确认正例失败，再按组件/参数种类校验。原 foo=%2G 拒绝目标仍保留；host 的百分号禁用和其他完整字符规则仍待后续，不将语法接受外推为传输支持。官方 errata 索引抓取失败，已用包含 verified errata 的官方辅助视图对照现有规则，未声称穷尽所有 RFC 更新。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 不完整百分号转义测试复核通过：用户新增 `rejects_sip_uri_with_incomplete_percent_escape`，同一测试分别输入 `sip:bob%@example.com` 与 `sip:bob%2@example.com`，均完整比较百分号转义无效错误，覆盖缺两位和缺一位。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：40 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `rejects_sip_uri_with_invalid_percent_escape_in_parameter_value`，输入 `sip:bob@example.com;foo=%2G`，完整比较 Err(String::from("URI 百分号转义无效"))。已核对 RFC 3261 §25 的 other-param/pvalue/paramchar，普通扩展参数值中的 %2G 也不合法；当前仅验证 userinfo，静态推导新输入会被接受，故新断言预计失败，尚未创建或运行。先由用户补此用例，随后可将现有扫描提取为小校验函数供实际的 userinfo 与参数调用复用，仍保存原文；不提前实现其他组件。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整 URI 仍未完成，未提交或推送。

2026-10-27 userinfo 百分号格式校验复核通过：用户已用 bytes 可变迭代器替换 contains 空块，遇 % 读取两位，以 is_some_and/is_ascii_hexdigit 拒绝缺失或非法十六进制字节，仍保存 userinfo 原文。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak` 退出 0：52 passed、0 failed、0 ignored、0 filtered out，含 39 项 URI、11 项起始行、2 项 CLI；非法 %2G 已拒绝、合法 %20 原文测试通过。本轮 test target 无警告，fmt 与 git diff --check 均退出 0，此前格式与尾空白问题已消除。

下一步建议用户新增 `rejects_sip_uri_with_incomplete_percent_escape`，同一测试内分别输入 `sip:bob%@example.com` 和 `sip:bob%2@example.com`，每组完整比较 Err(String::from("URI 百分号转义无效"))，覆盖 userinfo 末尾缺两位及缺一位的 None 分支。按当前实现静态推导应直接通过，尚未创建或运行此用例。此步只覆盖 userinfo 的转义格式，其余组件与完整字符规则仍待后续。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；Python/CLI 集成、Clippy、release、完整 check.sh、远端 CI 未运行，未提交或推送。

2026-10-27 百分号校验片段讲解与第 01 章进度核对：用户已新增 `rejects_sip_uri_with_invalid_percent_escape`，输入 `sip:bo%2Gb@example.com`，完整比较预期错误，但实现仅在 userinfo 分离后的 contains('%') 空块中停住。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak` 退出 101：51 passed、1 failed、0 ignored、0 filtered out，唯一失败为新用例实际返回 Ok(userinfo=bo%2Gb)；其余 38 项 URI、11 项起始行、2 项 CLI 均通过。编译有额外 if 括号的 unused_parens 警告；fmt 退出 1（空块格式），git diff --check 退出 2（src/main.rs:130 尾部空白），不将这些检查报为通过。

仅会话提供局部思路和示意：用 userinfo.bytes() 可变迭代器直接替换整个 contains 空块，while let Some 逐字节读取，遇 % 再 next 两次，借助 Option::is_some_and 与 is_ascii_hexdigit 确认两字节均存在且为十六进制，任一不满足返回既定错误；继续保存原文，不解码。独立只读复核认可，示意代码尚未写入或实际验证。下一步由用户完成此局部校验、格式化并复跑，再补 %、%2 等不完整边界。

按 targets.json 与 progress.json 核对：第 01 章 4 个 AC 中，AC-01-01 为 in_progress，02/03/04 为 not_started，尚无正式 passed。本项学习顺序为起始行→URI→消息头→Via/Contact→完整消息/body/序列化；当前到第二步，URI 的 ?headers 不等于消息头，后续还需完成模型与解析/序列化，再验证字节边界、资源上限/语料模糊及往返独立验证。不以测试数量或在做 1/4 推算完成百分比。本轮仅更新 CURRENT 与任务卡，不改源码、测试、需求或进度；Python/CLI 集成、Clippy、release、完整 check.sh、远端 CI 未运行，未提交或推送。

2026-10-27 百分号转义 userinfo 原文保留复核通过：用户新增 `parse_sip_uri_with_percent_encoded_userinfo`，输入 `sip:bo%20b@example.com`，完整比较 scheme=sip、userinfo=Some(bo%20b)、host=example.com、port=None、空参数与 headers；静态及独立只读复核认可。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：38 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `rejects_sip_uri_with_invalid_percent_escape`，输入 `sip:bo%2Gb@example.com`，完整比较 Err(String::from("URI 百分号转义无效"))。已核对 RFC 3261 §25.1 的 escaped="%" HEXDIG HEXDIG，userinfo 中 % 后的 2G 含非法 G；当前无对应校验，静态推导返回 Ok，新断言预计失败，尚未创建或运行。后续再验证不完整的 %、%2，按组件只验证格式并保留原文，不外推 host 可转义。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整字符和转义校验仍未完成，未提交或推送。

2026-10-27 空格及控制字符拒绝复核通过：用户已用 `input.bytes().any(|b| b == b' ' || b.is_ascii_control())` 替换原 contains 检查，错误文案统一为「URI 不能包含空白或控制字符」。原空格测试合并为 `rejects_sip_uri_with_unescaped_control_character`，在同一函数内分别完整比较普通空格和 NUL 两组输入的错误。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：37 passed、0 failed、0 ignored、13 filtered out；测试函数总数仍为 37，fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `parse_sip_uri_with_percent_encoded_userinfo`，输入 `sip:bo%20b@example.com`，完整比较 scheme=sip、userinfo=Some(bo%20b)、host=example.com、port=None、uri_params=[]、uri_headers=[]，保护合法百分号转义的原文保留。当前逻辑静态推导应直接通过，该测试尚未创建或运行；随后再验证非法 %HH 格式。入口字节检查与两组已测输入不等于完整字符/转义校验。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 原始空格拒绝测试及写法评审：用户在 parse_sip_uri 开头增加 input.contains(' ')，新增实际名 `parse_sip_uri_with_unescaped_space`，输入 `sip:bo b@example.com`，完整比较 Err(String::from("URI 不能包含空白字符"))。静态及独立只读复核认可当前空格目标与检查位置。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：37 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

用户询问更好写法，会话仅建议扩为 `input.bytes().any(|b| b == b' ' || b.is_ascii_control())`，覆盖原始空格及 ASCII 控制字节，并将文案与现有断言同步改为「URI 不能包含空白或控制字符」。已核对 RFC 3261 §19.1.2 与 Rust u8 文档；is_ascii_whitespace 单独会漏 NUL/DEL。当前 `sip:bo\0b@example.com`（Rust 字符串中的 NUL）静态推导仍会被接受，尚未创建或运行对应测试。下一步由用户补 `rejects_sip_uri_with_unescaped_control_character`，以该输入完整比较新错误，再修正判断及原空格断言；之后继续 %20 原文保留。建议代码与新测试均未代写，完整字符与转义校验仍未完成。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 混合大小写 scheme 测试复核通过：用户新增 `parse_sip_uri_with_mixed_case_scheme`，输入 `sIpS:pop@example.com`，完整比较 scheme 原样 sIpS、userinfo=Some(pop)、host=example.com、port=None、空参数与 headers；静态及独立只读复核认可。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：36 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `rejects_sip_uri_with_unescaped_space`，输入 `sip:bo b@example.com`，完整比较 Err(String::from("URI 不能包含空白字符"))。已核对 RFC 3261 §19.1.2 明确禁止 URI 中未转义的空格与控制字符；当前无对应检查，静态推导会将 bo b 存为 userinfo 并返回 Ok，新断言预计失败，但尚未创建或运行。先由用户覆盖原始空格拒绝，再用 %20 原文保留测试及 %HH 格式校验继续学习；本例不代表完整字符验证。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整 URI 验收仍未完成，未提交或推送。

2026-10-27 sips 正常样例复核通过：用户新增 `parse_sip_uri_with_sips_scheme`，实际输入 `sips:pop@example.com`，完整比较 scheme=sips、userinfo=Some(pop)、host=example.com、port=None、uri_params=[]、uri_headers=[]；静态及独立只读复核认可，pop 与输入一致。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：35 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `parse_sip_uri_with_mixed_case_scheme`，输入 `sIpS:pop@example.com`，完整比较 scheme=sIpS，其余字段同该 sips 样例，验证忽略大小写识别并保留原始文本。按当前 eq_ignore_ascii_case 与 scheme.to_string() 流程静态推导应直接通过，该用例尚未创建或运行。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，完整 URI 验收仍未完成，未提交或推送。

2026-10-27 scheme 拒绝条件修正复核通过：用户已改为 `!scheme.eq_ignore_ascii_case("sip") && !scheme.eq_ignore_ascii_case("sips")`，同时不匹配才拒绝，原 scheme 文本仍保存；静态及独立只读复核认可。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：34 passed、0 failed、0 ignored、13 filtered out；此前 31 项回归均消除，新 http 拒绝测试也通过。fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `parse_sip_uri_with_sips_scheme`，输入 `sips:bob@example.com`，完整比较 scheme=sips、userinfo=Some(bob)、host=example.com、port=None、uri_params=[]、uri_headers=[]。无显式端口保持 None，按当前代码静态推导应直接通过；尚未创建或运行该用例。随后再单独验证混合大小写 scheme。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；SIPS 此步只验证 URI 语法，13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 scheme 拒绝条件回归诊断：用户新增实际名 `rejects_sip_uri_with_unsupoorted_scheme`，输入 `http:bob@example.com`，完整比较不支持 scheme 的错误，该项通过；但新条件 `scheme != "sip" || scheme != "sips"` 恒为 true，使所有非空 scheme 提前返回同一错误。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101：3 passed、31 failed、0 ignored、13 filtered out；31 项均因实际 Err("不支持的 URI scheme") 与原预期不同而失败，非编译错误。fmt 与 git diff --check 均退出 0。

静态及独立只读复核的修正建议为 `!scheme.eq_ignore_ascii_case("sip") && !scheme.eq_ignore_ascii_case("sips")`：同时不匹配两种允许值才拒绝，保持原 scheme 字符串。以 sip 为例，旧式 false || true 为 true，误进错误分支；新式 false && true 为 false。现有失败输出与布尔式已直接定位同一根因，省略多假设及插桩。仅会话讲解并同步 CURRENT/任务卡，不代改源码或测试；由用户修改此条件再跑 URI 回归，尚未执行修正后验证。13 项起始行/CLI 单元测试及完整工程检查未运行，进度不变，未提交或推送。

2026-10-27 空 scheme 测试复核通过：用户在首个冒号拆分后、构造 SipUri 前增加 scheme.is_empty() 检查，并新增 `rejects_sip_uri_with_empty_scheme`，输入 `:bob@example.com`，完整比较 Err(String::from("scheme 不能为空"))；静态及独立只读复核认可检查位置与断言。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：33 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `rejects_sip_uri_with_unsupported_scheme`，输入 `http:bob@example.com`，完整比较 Err(String::from("不支持的 URI scheme"))。当前仅检查非空，静态推导会返回 Ok，故新断言预计失败；尚未创建或运行该用例。已核对 RFC 3261 §25、§19.1.4，建议本函数识别 sip/sips、按 ASCII 忽略大小写比较，并保留原始 scheme 文本；随后分别补 sips 与混合大小写正常样例，不外推 TLS 或所有 Request-URI 的限制。本轮只更新工作记录，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。辅助搜索误用了不存在的 docs/adr，rg 退出 2；已定位实际 docs/architecture/adr 并搜索，未发现相关 scheme 约定，未将搜索失败记为测试失败。

2026-10-27 userinfo 分隔符测试复核通过：用户新增实际名 `parse_sip_uri_with_delimiters_inuserinfo`，输入 `sip:bob;tag?x@example.com;transport=udp?subject=hello`，完整比较 userinfo=Some(bob;tag?x)、host=example.com、参数 transport=udp 和 header subject=hello，其余字段也与输入一致；静态及独立只读复核认可断言。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：32 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `rejects_sip_uri_with_empty_scheme`，输入 `:bob@example.com`，完整比较 Err(String::from("scheme 不能为空"))。已核对 RFC 3986 §3.1 的 scheme 非空规则；当前 split_once(':') 只确认冒号存在，未拒绝其前空字符串，静态推导新断言会失败，该用例尚未创建或运行。先由用户写测试，再于拆分后补最小空值检查；这一步不代表已验证允许的 scheme、字符或大小写规则。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 参数与 headers 组合复核通过：用户新增 `parse_sip_uri_with_parameters_and_headers`，输入 `sip:bob@example.com;transport=udp;lr?subject=hello&priority=urgent`，完整比较正常 host、port=None、有序参数 transport/lr 和有序 headers subject/priority；静态及独立只读复核认可。空 header 名测试的 sup 拼写已修正为 sip。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：31 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

下一步建议用户新增 `parse_sip_uri_with_delimiters_in_userinfo`，输入 `sip:bob;tag?x@example.com;transport=udp?subject=hello`，完整比较 scheme=sip、userinfo=Some(bob;tag?x)、host=example.com、port=None、uri_params=[(transport, Some(udp))]、uri_headers=[(subject, hello)]。已核对 RFC 3261 §25 的 user-unreserved 包含分号和问号，本例保护先拆 userinfo 再拆问号/分号的组件顺序，避免将用户名里的合法字符误当分隔符。按现逻辑静态推导应直接通过，该测试尚未创建或运行。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 空 header 名称测试复核：用户新增实际名 `rejects_sup_uri_with_empty_header_name`，输入 `sip:bob@example.com?=hello`，完整比较空 header 名称错误；静态及独立只读复核认可断言。名称中的 sup 应改为 sip，否则常用 sip_uri 过滤不会运行这项测试。本轮因此使用无过滤命令 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak`，退出 0：43 passed、0 failed、0 ignored、0 filtered out，包含该用例及其他 29 项 URI、11 项起始行、2 项 CLI 单元测试。fmt 与 git diff --check 均退出 0，不将拼写导致的漏筛选风险当作测试失败。

下一步建议由用户新增 `parse_sip_uri_with_parameters_and_headers`，输入 `sip:bob@example.com;transport=udp;lr?subject=hello&priority=urgent`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=None，参数顺序 [(transport, Some(udp)), (lr, None)]，headers 顺序 [(subject, hello), (priority, urgent)]，确认两段边界协作。按当前逻辑静态推导应直接通过，该用例尚未创建或运行。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；未运行 Python/CLI 集成回归、Clippy、release 或完整 check.sh，未提交或推送。

2026-10-27 header 缺等号测试复核通过：用户新增 `rejects_sip_uri_with_header_without_equals`，输入 `sip:bob@example.com?subject`，完整比较 Err(String::from("header 必须包含等号"))，对应 None 分支错误文案已修正。静态及独立只读复核认可该拒绝路径，合法空值规则保持。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：29 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0。

下一步建议由用户新增 `rejects_sip_uri_with_empty_header_name`，输入 `sip:bob@example.com?=hello`，完整比较 Err(String::from("header 参数名不能为空"))，验证已有 name.is_empty() 分支。按现有逻辑静态推导应直接通过，该用例尚未创建或运行。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-27 多 header 测试复核通过：用户新增 `parse_sip_uri_with_multiple_headers`，输入 `sip:bob@example.com?subject=hello&priority=urgent`，完整比较正常 scheme/userinfo/host、port=None、uri_params=[]、有序 headers=[(subject, hello), (priority, urgent)]。静态及独立只读复核认可断言。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：28 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0。

下一步建议由用户新增 `rejects_sip_uri_with_header_without_equals`，输入 `sip:bob@example.com?subject`，完整比较 Err(String::from("header 必须包含等号"))。当前缺等号已被拒绝，但内层 None 文案误为 header 参数名不能为空，因此静态推导新测试会先因错误文案不一致失败；仅将该 None 分支改为明确缺等号，不放宽语法。此用例尚未创建或运行，不把文案问题误报为接受非法输入。随后再验证 header 空名称；其余组合、字符及转义边界仍待后续。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 空 header 值测试修正复核通过：用户已将输入修正为 `sip:bob@example.com?subject=`，完整比较正常 host 与 uri_headers=[(subject, 空字符串)]，静态及独立只读复核认可断言。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：27 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0。之前误写输入导致的失败已消除，历史记录保留。

下一步建议由用户新增 `parse_sip_uri_with_multiple_headers`，输入 `sip:bob@example.com?subject=hello&priority=urgent`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=None、uri_params=[]、uri_headers 按顺序为 [(subject, hello), (priority, urgent)]，验证 & 拆项及顺序。现有逻辑静态推导应直接通过，本轮尚未创建或运行该用例。缺等号的错误文案及其他 headers/转义边界仍待后续，不将当前样例视为完整 URI 验收。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 空 header 值测试输入诊断：用户已删除 header 的空值拒绝并新增 `parse_sip_uri_with_empty_header_value`，但实际输入误写为 `sip:bob@example.com?subjet`，缺少等号且 subject 少 c，期望却是 (subject, 空字符串)。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak tests::parse_sip_uri_with_empty_header_value -- --exact` 退出 101：0 passed、1 failed、0 ignored、39 filtered out；编译成功，实际因 split_once('=') 返回 None 而得到 Err("header 参数名不能为空")，与期望 Ok 不同。

静态及独立只读复核认可最小修正：只把测试输入改为 `sip:bob@example.com?subject=`；期望中的 String::from("") 正确，不必更换。None 分支表示缺少等号，当前错误文案不准确，可明确为缺等号；不通过放行缺等号输入来满足空值测试。正确输入的通过结果仍仅为静态推导，未代改或复跑。fmt 与 git diff --check 均退出 0。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；下一步由用户修正输入后复跑 URI 测试，其余 39 项及完整工程检查未运行，未提交或推送。

2026-10-26 headers 普通样例复核通过及空值边界：用户已修正闭包返回类型、Result 收集、分号回退值并写回 uri_headers。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：26 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0，此前两处 E0308 在本轮测试编译中已消除。

静态及独立只读复核发现 uri_headers 解析仍保留 value.is_empty() 拒绝，会错拒合法 `sip:bob@example.com?subject=`。下一步由用户只删除 header 解析中的该空值检查，并新增 `parse_sip_uri_with_empty_header_value`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=None、uri_params=[]、uri_headers=[(subject, 空字符串)]；按当前代码静态推导将失败，该用例尚未创建或运行。缺等号分支错误文案目前写成空值，也建议明确为缺少等号；此项不影响已通过普通样例的结论。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 headers 片段类型报错讲解：用户已在 userinfo 后分离问号，并尝试在本地 sip_headers 中使用 map 解析。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak tests::parse_sip_uri_with_header -- --exact` 退出 101，编译器实际报告两处 E0308：内层 Some 块后的 `?` 将 Ok(tuple) 拆成 tuple，与闭包 return Err 所要求的 Result 冲突；外层 Some 只返回 map 迭代器，None 返回 Vec，缺少收集步骤。测试执行 0 项，未进入断言；fmt 与 git diff --check 均退出 0。

静态及独立只读复核建议：保留本地名 sip_headers，去掉内层块后的问号，闭包成功返回 Ok(tuple)、缺等号返回 Err，再于 map 后使用 `.collect::<Result<Vec<_>, String>>()?`；header 值允许为空，应去掉空值拒绝。另需将分号拆分的 None 改为 (hostport_and_params, None)，避免重新使用含问号的原始 host，并在末尾写回 sipuri.uri_headers=sip_headers。会话仅给局部示意及返回类型解释，不代改源码或测试。下一步由用户修正后重跑 header 单项和 URI 回归；字符与转义等后续边界仍未完成。本轮仅更新 CURRENT 与任务卡，进度不变，未运行完整工程检查，未提交或推送。

2026-10-26 URI headers 模型与首测复核：用户已新增 uri_headers 字段并同步原有全部构造，同时提前新增 `parse_sip_uri_with_header`，输入 `sip:bob@example.com?subject=hello`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=None、uri_params=[]、uri_headers=[(subject, hello)]。静态及独立只读复核认可字段、构造与断言；当前尚未分离或写回 headers。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 101：25 passed、1 failed、0 ignored、13 filtered out。新用例实际 host="example.com?subject=hello"、uri_headers=[]，已复现预期的新行为缺失；原有 25 项通过。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0。下一步由用户在 userinfo 分离后、分号切分前按首个问号拆出 hostport_and_params 与 headers_text；前者沿现有参数/host/port 路径，后者按 & 逐项、首个 = 拆名值，拒绝缺等号与空名称，允许空值，用 Result collect 汇总后写回 uri_headers。没有问号保持空 Vec，存在空 headers 不能直接当无 headers；完成所有借用后再覆盖 host。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 IPv6/端口/参数组合测试复核：用户新增实际名 `parse_sip_uri_with_ipv6_port_and_parmeters`，输入 `sip:bob@[2001:db8::1]:5056;transport=udp;lr`，完整比较 host 保留方括号、port=Some(5056)、有序参数 transport=udp 与 lr=None；输入和期望一致，静态及独立只读复核认可。函数名 parmeters 可顺手改为 parameters，不影响行为。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：25 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。

下一小步进入 URI ?headers 模型，仅建议用户给 SipUri 加 `uri_headers: Vec<(String, String)>`，解析器初始化及 9 处正常测试期望共 10 处构造补 `uri_headers: Vec::new()`，先保持解析行为不变，不新增测试。已核对 RFC 3261 §25：header 必须含等号，名称非空而值可为空，因此值用 String，不沿用参数值的 Option 或非空检查。字段步骤完成后再写 `parse_sip_uri_with_header`，输入 `sip:bob@example.com?subject=hello`；后续在 userinfo 分离后、参数分离前确定问号边界，避免误切用户名中的问号。本轮未实现 headers，仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 参数名大小写判重复核通过：用户已改用 seen HashSet 与 insert(name.to_ascii_lowercase())，并新增 `rejects_sip_uri_with_duplicate_parameter_ignoring_case`，输入 `sip:bob@example.com;Transport=udp;transport=udp`，完整比较重复名错误。静态及独立只读复核认可实现和测试，同值不影响该用例验证大小写判重，不要求为换值而阻塞；原参数 Vec 的大小写及顺序保持。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：24 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0。

下一步建议由用户新增组合测试 `parse_sip_uri_with_ipv6_port_and_parameters`，输入 `sip:bob@[2001:db8::1]:5060;transport=udp;lr`，完整比较 scheme=sip、userinfo=Some(bob)、host=[2001:db8::1]、port=Some(5060)、uri_params=[(transport, Some(udp)), (lr, None)]，确认参数切分不污染 IPv6 或端口边界。当前逻辑静态推导应直接通过，本轮尚未创建或运行；随后再学习 URI 的 ?headers。百分号转义及完整参数语法仍待后续，不将当前测试视为完整 URI 验收。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 重复参数校验评审：用户通过“参数名克隆到 HashSet 后比较数量”拒绝完全同名的重复，新增 `rejects_sip_uri_with_duplicate_parameter` 使用两个 transport=udp，完整比较重复名错误，该测试有效。空值测试函数名的 parameter 拼写已修正，输入仍带额外末尾分号，不影响当前先触发空值错误的断言。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：23 passed、0 failed、0 ignored、13 filtered out；fmt 与 git diff --check 均退出 0。

静态及独立只读复核发现：原样 HashSet<String> 区分大小写，会漏掉 `sip:bob@example.com;transport=udp;Transport=tcp`。已核对 RFC 3261 §19.1.4 的大小写比较规则及 HashSet::insert 官方文档。会话仅建议保留 HashSet，改用本地 seen 集合与 for 循环，将 name.to_ascii_lowercase() 作为比较键，insert 返回 false 即报重复错误；原参数 Vec 的大小写与顺序保留，不必再比较计数。如保留原方式，长度应直接用 len() 表达。当前阶段只讨论原始 ASCII 参数名，百分号转义等后续处理仍未完成。

下一步由用户修改判重，并新增 `rejects_sip_uri_with_duplicate_parameter_ignoring_case`，输入上述大小写混合样例，完整比较 Err(String::from("URI 参数名不能重复"))；也建议既有重复测试改用 udp/tcp 不同值，明确按名称判重。新样例尚未创建或运行，不将静态判断记为已复现失败。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 空参数值测试复核：用户新增实际名为 `rejects_sip_uri_with_empty_paramter_value` 的测试，输入 `sip:bob@example.com;transport=;`，完整比较空值错误。静态及独立只读复核认可断言：首项 transport= 即触发空值错误，Result 收集提前结束，末尾空项未被读取；建议用户将名称中的 paramter 拼写改为 parameter，并去掉输入末尾额外分号，使样例聚焦空值。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：22 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。

下一步建议由用户新增 `rejects_sip_uri_with_duplicate_parameter`，输入 `sip:bob@example.com;transport=udp;transport=tcp`，完整比较 Err(String::from("URI 参数名不能重复"))。已核对 RFC 3261 §19.1.1，同一参数名不可出现多次；错误文案为本项目拟定。当前没有重复名检查，静态推导新断言会失败，尚未创建或执行该用例；后续还需验证大小写等其余边界。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 末尾参数分号测试复核：用户新增 `rejects_sip_uri_with_trailing_parameter_separator`，输入 `sip:bob@example.com;transport=udp;`，完整比较空名错误，并在内层无等号的 None 分支增加 item.is_empty() 检查。静态及独立只读复核认可该拒绝路径，非空 lr 仍保存为 None，未通过过滤空项规避错误。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：21 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0。

下一步建议由用户新增 `rejects_sip_uri_with_empty_parameter_value`，输入 `sip:bob@example.com;transport=`，完整比较 Err(String::from("URI 参数值不能为空"))，验证已实现但尚无专门测试的空值检查。根据现有 value.is_empty() 分支，该测试预计直接通过；本轮尚未创建或实际运行，不能冒称已经验证。无值 lr 的 None 与显式空值需区别；其余参数及 headers 边界仍待后续。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 参数校验错误传播复核通过：用户已将内层 Some/None 成功路径统一为 Ok，并使用 `.collect::<Result<Vec<_>, String>>()?` 汇总和传播错误；现有 `rejects_sip_uri_with_empty_parameter` 的 `sip:bob@example.com;=udp` 能返回预期空名错误。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：20 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0，此前 E0308 在本轮测试编译中已消除。

下一步建议由用户新增 `rejects_sip_uri_with_trailing_parameter_separator`，输入 `sip:bob@example.com;transport=udp;`，完整比较 Err(String::from("URI 参数名不能为空"))，验证不含等号的空参数项。当前末尾分号使 split 产生空 item，内层 None 分支仍直接返回 Ok(("", None))，静态推导新断言将失败；该用例尚未创建或运行。空值检查已有代码但尚无专门测试，其他参数及 headers 边界仍待后续。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 参数校验闭包类型错误讲解：用户新增实际名为 `rejects_sip_uri_with_empty_parameter` 的测试，输入 `sip:bob@example.com;=udp`，完整比较空名错误，断言符合目标；同时在 map 的 Some 分支增加空名及空值检查。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak tests::rejects_sip_uri_with_empty_parameter -- --exact` 退出 101，编译器实际报告两处 E0308：闭包中 return Err 推断返回 Result，但成功路径仍返回普通元组；普通 collect 及外层空 Vec 又使结果被推断成 Vec<Result<...>>，不能赋给参数字段。测试执行 0 项，未进入断言，不视为协议断言失败。

会话仅讲解局部修正，独立只读复核认可：保留现有空名/空值检查，将内层 Some、None 成功路径包成 Ok，末尾改为 `.collect::<Result<Vec<_>, String>>()?`，外层无参数的 None=>Vec::new() 保持。map 内 return 仅退出闭包；collect 汇总逐项 Result，? 才向 parse_sip_uri 传播错误。已核对 Rust Iterator::collect 官方文档。无等号分支空 item 仍为待测边界，不用过滤空项代替拒绝。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0。本轮只更新 CURRENT 与任务卡，不改源码、测试或进度；下一步由用户完成上述三处修改再复跑 URI 测试，未运行完整工程检查，未提交或推送。

2026-10-26 URI 多参数测试复核：用户新增 `parse_sip_uri_with_multiple_parameters`，输入 `sip:bob@example.com;transport=udp;lr`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=None，参数按顺序为 [(transport, Some(udp)), (lr, None)]。静态及独立只读复核认可断言，Vec 的完整比较确实检查顺序及无值参数的 None。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：19 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。

下一步建议由用户新增 `rejects_sip_uri_with_empty_parameter_name`，输入 `sip:bob@example.com;=udp`，完整比较 Err(String::from("URI 参数名不能为空"))。已核对 RFC 3261 §25 的 pname=1*paramchar，空参数名不合法；错误文案为本项目拟定。当前 map 没有空名校验，静态推导将接受 ("", Some("udp"))，故新断言预计失败，但本轮尚未创建或实际执行该用例。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 URI 参数正常样例复核通过：用户已修正 match/map/collect 的返回值与 String 转换，移除中间重复 collect，并把结果写入 sipuri.uri_params；无显式类型标注也可由最终字段赋值推断。静态及独立只读复核认可当前 transport=udp 正常样例与既有无参数路径。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：18 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check`、`git diff --check` 均退出 0。此前语法错误和尾部空白已在本轮检查中消除，历史失败记录保留。

下一步建议由用户新增 `parse_sip_uri_with_multiple_parameters`，输入 `sip:bob@example.com;transport=udp;lr`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=None、uri_params 按顺序为 [(transport, Some(udp)), (lr, None)]，验证多项参数及无值参数；尚未创建或运行该测试。空参数、重复参数、URI headers 等边界仍待后续学习，不将当前正常样例视为完整 URI 验收。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度；13 项起始行/CLI 单元测试及完整工程检查未运行，未提交或推送。

2026-10-26 URI 参数片段讲解及编译核对：用户已正确按首个分号分离 hostport 与 params_text，并尝试 match、split、map、collect。`RUSTUP_AUTO_INSTALL=0 cargo check --locked --offline --all-targets` 退出 101，编译器在 src/main.rs:137 实际报告两处比较运算符不能连用：表达式中的 `Vec<(String, Option<String>)>::new()` 缺少泛型前的 `::`；可写 `Vec::<...>::new()`，或给 sip_params 标注完整类型后使用 `Vec::new()`。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 同因该语法错误退出 1。测试执行 0 项，未将上轮结果沿用为本轮通过。

独立只读复核确认其余静态问题：Some 块以 `let a = ...collect();` 结束而返回 ()；map 产出借用字符串而模型要求拥有的 String；中间 collect 成 Vec 再 iter 没有必要，最终也尚未写回 uri_params。会话仅给保留 match+map 的局部示意：明确 Vec 类型、直接从 split 迭代、名值转 String、Some 分支直接返回 collect 表达式、None 返回空 Vec，末尾把 sip_params 写入 sipuri.uri_params。下一步由用户修正这一段、完成借用后写回并重跑 transport 单项及 URI 回归；当前只讲解无 headers 正常样例，参数错误边界仍待验证。本轮只同步 CURRENT 与任务卡，不改源码、测试或进度，未提交或推送。

2026-10-26 transport 参数测试失败确认：用户新增 `parse_sip_uri_with_transport_parameter`，输入 `sip:bob@example.com;transport=udp`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=None、uri_params=[(transport, Some(udp))]；静态及独立只读复核认可输入和期望。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak tests::parse_sip_uri_with_transport_parameter -- --exact` 退出 101：0 passed、1 failed、0 ignored、30 filtered out。实际 host="example.com;transport=udp"、uri_params=[]，实测确认尚未分离及解析 URI 参数；不调整正确断言。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。

下一步由用户在分离 userinfo 后、IPv6/端口处理前，按首个分号拆出 hostport 与可选参数文本；hostport 沿用现有路径，参数按分号逐项、首个等号拆名值并转为拥有的字符串，保留顺序和无值参数的 None。无参数保持空 Vec，避免拆出伪空项；host 与参数借自 sipuri.host，应完成所有借用及转换后再写回。本轮讲解仅面向无 headers 的正常样例，空参数等边界后续明确验证，不静默丢弃。本轮只同步 CURRENT 与任务卡，不改源码、测试或进度；其余 30 项测试及完整工程检查未运行，未提交或推送。

2026-10-26 URI 参数模型完成复核：用户已新增 `uri_params: Vec<(String, Option<String>)>`，解析器初始化及 6 处正常测试期望共 7 处构造均补齐 `Vec::new()`；静态及独立只读复核确认，目前仍无参数解析逻辑。`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：17 passed、0 failed、0 ignored、13 filtered out；`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。

下一步由用户只新增 `parse_sip_uri_with_transport_parameter`，输入 `sip:bob@example.com;transport=udp`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=None、uri_params=[(transport, Some(udp))]。按当前代码静态推导，该用例将因 host 仍包含 `;transport=udp`、uri_params 为空而断言失败，用于明确接下来的解析目标；尚未创建或实际运行，不能记作已复现失败。本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度；未运行完整工程检查，未提交或推送。

2026-10-23 URI 参数模型学习安排：用户要求“继续”，沿用用户亲自编写、代理讲解与复核的边界。本步建议仅给 SipUri 增加 `uri_params: Vec<(String, Option<String>)>`，解析器初始化及 6 处正常测试期望共 7 处构造补 `uri_params: Vec::new()`；不修改解析行为、不新增测试。Vec 保存参数顺序及未知扩展，Option 表达 `transport=udp` 的 Some("udp") 与无值 `lr` 的 None，依据 RFC 3261 §19.1.1、§25。之后再以 `parse_sip_uri_with_transport_parameter` 学习解析 `sip:bob@example.com;transport=udp`，本步尚未创建或执行该用例；分离参数应在 userinfo 处理后，不能全局按分号切整条 URI。

本轮仅更新 CURRENT 与任务卡，不改源码、测试或进度，Rust 测试执行 0 项；最近一次 URI 结果仍为 17 passed、13 filtered out，非本轮重跑。下一步由用户完成字段与构造更新，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 检查兼容性；未运行完整工程检查，未提交或推送。

2026-10-23 IPv6 错误测试修正复核：用户已将非法尾部期望修正为 Err(String::from("IPv6 闭括号只能接端口"))，空地址期望修正为 Err(String::from("IPv6 地址无效"))。静态及独立只读复核确认五项测试的名称、输入与完整错误断言符合原定意图，缺闭括号、非法尾部、空地址、非法地址、空端口均已覆盖，无缺项；该结论仅覆盖本组用例，不代表完整 URI 验收。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：17 passed、0 failed、0 ignored、13 filtered out；起始行/CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。本轮仅同步 CURRENT 与任务卡，不改源码、测试或进度，不覆盖上轮 2 项失败记录；未运行完整 check.sh，未提交或推送。下一步由用户安排剩余 URI 组件及边界学习，当前整项仍为 in_progress。

2026-10-23 IPv6 五项错误测试复核：用户已补齐缺闭括号、非法尾部、空地址、非法地址与空端口测试。缺闭括号测试输入现为 `sip:bob@[::1`，已通过；此前中断轮次最初读取的输入缺少 `@`，随后实际执行单项时已被用户修正并通过，不将未捕获的旧失败当作本轮复现。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 本轮退出 101：17 项中 15 passed、2 failed、0 ignored、13 filtered out。两处失败为测试期望错误：`rejects_sip_uri_with_invalid_ipv6_suffix` 的 `sip:bob@[::1]abc` 应为 Err(String::from("IPv6 闭括号只能接端口"))，因尾部未以冒号开头，在端口数字检查前返回；`rejects_sip_uri_with_empty_ipv6_address` 的 `sip:bob@[]` 应为 Err(String::from("IPv6 地址无效"))，括号内部空字符串在 Ipv6Addr 校验处失败，整个 host 文本 `[]` 本身并非空。独立只读复核认可这两个样例的解析行为及其余三项错误断言。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。

下一步由用户仅修正上述两处期望，再复跑 URI 测试；本轮不代改源码或测试，只更新 CURRENT 与任务卡。状态保持 in_progress，不将当前失败记为通过；未运行完整工程检查，未提交或推送。

2026-10-23 IPv6 正常测试及命名建议：用户已编写 IPv6 分支与两个正常测试，实际名称为 `parse_sip_uri_with_ipv6`、`parse_sip_uri_with_ipv6_port`，输入分别为 `sip:bob@[2001:db8::1]` 与 `sip:bob@[2001:db8::1]:5056`，完整比较 host 保留方括号、port 为 None / Some(5056)。静态及独立只读复核认可这两个断言；按用户请求仅建议更明确的名称 `parse_sip_uri_with_ipv6_without_port`、`parse_sip_uri_with_ipv6_and_port`，不代改源码或测试。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri` 退出 0：12 passed、0 failed、0 ignored、13 filtered out；12 个 URI 测试函数覆盖 15 组输入，其余起始行/CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。下一步由用户补 IPv6 错误测试，先写缺闭括号的 `rejects_sip_uri_with_ipv6_missing_closing_bracket`，输入 `sip:bob@[::1`，完整比较 Err(String::from("IPv6 缺少闭括号"))；其后补非法尾部、空/非法地址及空端口。本轮只更新 CURRENT 与任务卡，进度状态不变；尚未实际验证这些 IPv6 错误分支，未运行完整工程检查，未提交或推送。

2026-10-23 IPv6 URI 解析讲解：用户询问如何扩充 `parse_sip_uri`，本次仅讲解，不视为代理实现授权。当前直接 `split_once(':')` 会切入 IPv6 地址；建议先按方括号边界拆出 `(&str, Option<&str>)`，再统一复用非空 ASCII 十进制及 u16 端口检查，最后写回拥有的 host 与 port。`[` 开头必须找到 `]`，括号后为空表示无端口，否则须以 `:` 开头；host 沿用保留方括号的约定，括号内部用标准库 `Ipv6Addr` 校验，避免接受 `[]` 或非法地址。非括号路径保留现有域名/IPv4 拆分。

已核对 RFC 3261 §25、RFC 5954 §4.1 与 Rust `Ipv6Addr` 文档，独立只读复核认可分支边界、校验复用及借用顺序。下一步由用户编写该局部逻辑并补 IPv6 有/无端口、缺闭括号、非法尾部、空/非法地址与端口边界用例。此方案面向当前尚无 URI 参数与 headers 的练习，不代表完整 SIP URI 已支持。本轮只同步 CURRENT 与任务卡，不改源码、测试或进度；测试执行 0 项，未编译或运行所示方案，未提交或推送。记录检查 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项结构有效，30 个生成视图一致，无空白错误。

2026-10-23 URI 最大端口测试复核：用户新增 `parse_sip_uri_with_maximum_port`，解析 `sip:bob@example.com:65535`，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=Some(65535)；静态及独立只读复核认可断言。

`RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 10 个 URI 测试函数，覆盖 13 组输入。随后运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak`，退出 0：23 passed、0 failed、0 ignored、0 filtered out，包含 10 项 URI、11 项起始行及 2 项 CLI 单元测试；本轮 test target 无警告。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。

下一小步由用户处理 IPv6 host/port 边界：hostport 以 `[` 开头时先定位 `]`，缺失时报错；闭括号后为空表示无显式端口，为 `:digits` 则复用现有非空 ASCII 数字及 u16 检查，其他尾部报错；非方括号输入保留当前域名/IPv4 路径。当前 String 模型约定 host 保留 `[2001:db8::1]` 的方括号；这是项目内部表示选择。`sip:bob@[2001:db8::1]` 的 port 为 None，追加 `:5060` 后为 Some(5060)。先确定 host 与可选端口文本、完成转换，再写回结构体，避免在仍需借用时修改来源。方括号边界处理本身不证明地址有效；如校验地址内容，可用标准库 `parse::<std::net::Ipv6Addr>()` 解析括号内部文本。参考 [RFC 3261 §25](https://www.rfc-editor.org/rfc/rfc3261.html#section-25)、[RFC 5954 §4.1](https://www.rfc-editor.org/rfc/rfc5954.html#section-4.1) 与 [Rust Ipv6Addr](https://doc.rust-lang.org/std/net/struct.Ipv6Addr.html)。

仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试。完整 URI 与其余验收仍未完成；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。

2026-10-23 URI 端口越界测试复核：用户新增实际名 parse_sip_uri_with_out_of_range_port，输入 sip:bob@example.com:65536，完整比较 Err(String::from("port 超出 u16 范围"))；静态及独立只读复核认可断言，范围错误路径现已实际验证。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 9 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：9 passed、0 failed、0 ignored、13 filtered out。9 个 URI 测试函数覆盖 12 组输入；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。过程被中断后再次核对工作区及测试定义，尚无 65535 成功测试；不重复运行无新变化的测试。

下一小步只由用户新增 `parse_sip_uri_with_maximum_port`，解析 sip:bob@example.com:65535，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=Some(65535)，验证当前 u16 端口模型的合法上界；之后再继续 IPv6，不将当前端口练习视为完整 URI 支持。

仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文 65536 越界用例尚待新增的说明为此前安排。

2026-10-23 URI 端口格式测试复核：用户新增的实际函数名为 parse_sip_uri_with_invalid_port_format，分别验证空端口、abc、+505，均完整比较 Err(String::from("port 必须是十进制数字"))。静态及独立只读复核认可三个断言，+505 与此前建议 +5056 属于同一非法格式，无需修改数字。首次按建议的 rejects_ 函数名搜索无匹配、退出 1；随后根据测试清单定位实际 parse_ 名称，非测试失败。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 8 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：8 passed、0 failed、0 ignored、13 filtered out。8 个 URI 测试函数覆盖 11 组输入；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户新增 `rejects_sip_uri_with_out_of_range_port`，调用 parse_sip_uri("sip:bob@example.com:65536")，完整比较 Err(String::from("port 超出 u16 范围"))。该输入格式合法、数值刚超过 u16 上限 65535，用于验证范围错误路径；完整 URI 其他组成及其余端口边界仍按原计划逐步完成。

仅同步记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文端口格式测试尚待新增的说明为此前安排；不将现有简单 URI 测试视为完整语法或整项验收。

2026-10-23 带端口空 host 反例测试复核：用户新增独立的 parse_sip_uri_with_empty_host，输入 sip:bob@:5056，完整比较 Err(String::from("host 不能为空"))，旧 rejects_sip_uri_with_empty_host 中的 sip:bob@ 与 sip: 仍保留。静态及独立只读复核认可行为覆盖，独立函数写法可以保留，不要求合并。此前只静态推导的带端口空 host 反例现已实际验证。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 7 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：7 passed、0 failed、0 ignored、13 filtered out。7 个 URI 测试函数覆盖 8 组输入；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户新增 `rejects_sip_uri_with_invalid_port_format`，在同一测试函数分别验证 sip:bob@example.com:、sip:bob@example.com:abc、sip:bob@example.com:+5056，均完整比较 Err(String::from("port 必须是十进制数字"))。三个输入验证同一条非空 ASCII 十进制格式规则，可沿用用户的 let result 遮蔽写法；范围错误及完整 URI 其他组成后续逐步补齐。

仅同步记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文带端口空 host 反例尚待新增的说明为此前安排；完整 URI 支持及整项验收仍未完成。

2026-10-23 显式端口 URI 正常测试复核：用户新增 parse_sip_uri_with_port，实际输入为 sip:bob@example.com:5056，完整比较 scheme=sip、userinfo=Some(bob)、host=example.com、port=Some(5056)。静态及独立只读复核认可断言，5056 与输入一致，正常样例无需强制改为此前示意的 5060。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 6 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：6 passed、0 failed、0 ignored、13 filtered out。6 个 URI 测试函数覆盖 7 组输入，首次实际验证显式端口正常拆分；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户扩充现有 rejects_sip_uri_with_empty_host，保留 sip:bob@ 与 sip:，再加入 sip:bob@:5056，完整比较 Err(String::from("host 不能为空"))，实际验证前轮 host 检查移至端口拆分后的反例。仍是同一个测试函数；端口空值、格式、范围等边界及完整 URI 组成随后逐步补齐。

仅同步记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文尚无带端口测试的说明为此前状态；当前仅简单域名样例通过，不代表 IPv6/URI 参数/headers 等已支持。

2026-10-23 最终 host 检查位置调整复核：用户已把 host 非空检查移到端口 if let 之后、Ok(sipuri) 之前；静态及独立只读复核确认检查最终 host，上一轮拆分后空 host 漏检已在代码结构上修正。当前尚无显式端口测试或上述反例的新增断言，不把静态判断当成这些输入的实际验证。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 5 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：5 passed、0 failed、0 ignored、13 filtered out。仍是原 5 个 URI 测试函数、6 组无显式端口输入；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户新增 `parse_sip_uri_with_port`，调用 parse_sip_uri("sip:bob@example.com:5060")，完整比较手写 Ok(SipUri)：scheme 为 sip、userinfo 为 Some(String::from("bob"))、host 为 example.com、port 为 Some(5060)。保留原无端口测试，验证正常端口转换与 host 拆分。其余端口边界、IPv6、URI 参数/headers 及完整语法仍按原计划逐步学习，不将当前简单拆分视为完整 URI 支持。

仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文 host 检查仍在端口拆分前的说明为此前状态。

2026-10-23 URI 端口片段与最终 host 检查复核：用户将端口片段改成 split_once(':')、非空/ASCII 数字检查、u16 转换及先转换后写回字段，test target 编译成功，上轮 E0277 在本次测试编译中已消除。静态及独立只读复核确认数字检查和赋值顺序符合当前简单样例练习，但 host 非空检查仍在端口拆分之前；sip::5060 与 sip:bob@:5060 都会在拆分后得到空 host 而返回 Ok。新反例仅静态推导，未创建或执行测试。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 5 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：5 passed、0 failed、0 ignored、13 filtered out。仍为此前 5 个 URI 测试函数、6 组输入，没有带端口用例；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告，不外推普通 binary/Clippy 结果。

下一小步只由用户把现有 host 非空检查从 src/main.rs:119 移到端口 if let 完整结束后、Ok(sipuri) 之前，检查最终 host。完成后再逐步补带端口正常与边界测试；IPv6、URI 参数及 headers 的组件边界仍按原计划学习，当前拆分尚不代表完整 URI 支持。

仅同步记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文 E0277 未修复的说明保留为上一轮历史。

2026-10-22 URI 端口解析讲解及编译错误核对：用户已新增 port: Option<u16>，并把初始化及两个无端口正常测试期望同步为 None；同时在 parse_sip_uri 尾部尝试 contains(':')、split(':')、next()。用户询问如何继续。本轮 `RUSTUP_AUTO_INSTALL=0 cargo check --locked --offline --all-targets` 退出 101，实际报告 E0277：src/main.rs:127 的 ok_or_else 闭包返回 Err(String)，导致错误类型嵌套成 Result，问号无法转换为函数要求的 String。若保留迭代器写法，next 还要求 parts 可变；这是静态指出，编译器本轮只报告前述错误。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0。测试执行 0 项，当前修改未编译通过，不沿用之前 5 项 URI 测试结果声称本轮通过。

结合当前代码讲解：userinfo 处理后 sipuri.host 暂存 hostport；对简单域名/IPv4、无 URI 参数或 headers 的练习，可直接匹配 split_once(':')，拆出借用的 host 与 port_text，检查端口非空且全为 ASCII 十进制数字，转换为 u16 后再把拥有的 host 与 Some(port) 写回，避免仍需旧借用时修改来源。将共用 host 非空检查放到 host/port 拆分之后，以覆盖如 sip::5060 拆出空 host 的情况。以 sip:bob@example.com:5060 得到 host=example.com、port=Some(5060) 为本步目标；无冒号时保持 None。

仅在会话给局部示意，不代改源码或创建测试；上述冒号拆分不是完整 URI 解析法，IPv6 必须先识别方括号边界，URI 参数与 headers 也需后续按组件边界处理，不将学习子集当作永久限制或端口解析已完成。独立只读复核认可类型问题、借用顺序及教学边界。

仅同步记录及本项备注并 render 更新视图，in_progress 与验收字段不变；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下一步由用户修正该端口片段并完成简单样例解析，再逐步补其余规则和测试。

2026-10-22 缺 URI scheme 分隔符测试复核：用户新增 rejects_sip_uri_without_scheme_separator，调用 parse_sip_uri("bob@example.com") 并完整比较当前缺冒号错误；静态及独立只读复核认可断言。先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 5 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：5 passed、0 failed、0 ignored、13 filtered out。5 个 URI 测试函数覆盖 6 组输入，含有/无 userinfo 两正常样例、两个空 host 输入、空 userinfo 及缺冒号；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。这是当前简单样例与已约定边界覆盖，不代表完整 URI 语法完成。

下一小步由用户给 SipUri 增加 `port: Option<u16>`，同步现有三处构造（parse_sip_uri 初始化及两个正常测试期望）为 port: None。None 表示 URI 未显式提供端口，Some(5060) 将表达显式端口；本步仅扩充数据模型并保持原无端口样例，不计为端口解析已实现，下一步再学习 host/port 拆分。代理不代改类型、解析器或测试。

仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文缺冒号测试尚待新增的说明为此前状态。

2026-10-22 空 host 双输入测试复核：用户在现有 rejects_sip_uri_with_empty_host 中保留 sip:bob@ 并新增 sip:，采用两次 let result 与两条完整断言，并未改成循环。静态及独立只读复核确认变量遮蔽合法，当前写法可以保留；两个空 host 输入均实际验证。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 4 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：4 passed、0 failed、0 ignored、13 filtered out。这里仍是 4 个 URI 测试函数、5 组输入，空 host 测试含两个输入；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户新增 `rejects_sip_uri_without_scheme_separator`，调用 `parse_sip_uri("bob@example.com")`，完整比较当前原错误 `Err(String::from("`:` 符号不能为空"))`，验证缺少第一个冒号时返回 Err。其余完整 URI 组成与校验仍待逐步完成。

仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文 sip: 尚待加入或循环仍待完成的说明为此前安排，实际使用合法的变量遮蔽。

2026-10-22 相同期望的多输入测试讲解：用户问扩充用例时应替换原输入还是另建 result1。说明原 sip:bob@ 与新增 sip: 都应保留；因二者预期同一 host 错误，可在现有测试函数中用数组循环，每轮 let result 是本轮局部变量，无需给变量编号。示例仅在会话展示，并建议断言附带当前 input 便于定位失败。该写法仍是 1 个测试函数、2 个输入用例，不把输入数当成 cargo 的测试函数数量。独立只读复核认可讲解。

本轮仅讲解及同步 CURRENT/任务卡，源码仍是单输入空 host 测试；测试执行 0 项，未运行编译、格式或完整工程检查，不沿用上一轮结果声称新增 sip: 已验证。未改源码、测试、进度或需求，in_progress 与验收字段不变，未提交或推送。下一步仍由用户完成该测试扩充，再补缺冒号等用例。

2026-10-22 空 userinfo URI 测试复核：用户新增 rejects_sip_uri_with_empty_userinfo，调用 parse_sip_uri("sip:@example.com")，完整比较 Err(String::from("有'@'符号时 userinfo 不能为空"))。静态及独立只读复核认可该断言。先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 4 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：4 passed、0 failed、0 ignored、13 filtered out；11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户扩充现有 `rejects_sip_uri_with_empty_host`，保留 sip:bob@，再加入 sip:，两个输入均完整比较 Err(String::from("host 不能为空"))，覆盖有/无 @ 两条分支的空 host。无需新增重复测试函数或修改解析器；缺冒号用例随后再补，完整 URI 其他组成仍待学习。

仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文空 userinfo 测试尚待新增的说明为此前状态。

2026-10-22 空 host URI 测试复核：用户新增 rejects_sip_uri_with_empty_host，调用 parse_sip_uri("sip:bob@") 并完整比较 Err(String::from("host 不能为空"))，覆盖此前含 @ 分支漏检空 host 的位置。静态及独立只读复核认可该断言。先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 3 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：3 passed、0 failed、0 ignored、13 filtered out；本轮只执行 3 项 URI 测试，11 项起始行与 2 项 CLI 单元测试被过滤未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户新增 `rejects_sip_uri_with_empty_userinfo`，调用 `parse_sip_uri("sip:@example.com")`，完整比较 `Err(String::from("有'@'符号时 userinfo 不能为空"))`，验证有 @ 时左侧非空要求；无 @ 的合法 None 已由正常测试覆盖。其余 URI 边界与完整组成仍按学习顺序逐步补齐。

仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文空 host 测试尚待新增的说明为此前状态。

2026-10-22 无 userinfo URI 测试复核：用户新增 parse_sip_uri_without_userinfo，直接调用 parse_sip_uri("sip:example.com")，完整比较手写 Ok(SipUri)，包含 userinfo: None。静态及独立只读复核认可该断言。先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri -- --list` 列出 2 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak sip_uri`，退出 0：2 passed、0 failed、0 ignored、13 filtered out。本轮仅执行有/无 userinfo 两项 URI 正常测试，11 项起始行与 2 项 CLI 单元测试被过滤、未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户新增 `rejects_sip_uri_with_empty_host`，调用 `parse_sip_uri("sip:bob@")`，完整比较 `Err(String::from("host 不能为空"))`，验证含 @ 的输入也会经过共用 host 检查。其余缺分隔符、空字段用例及完整 URI 组成仍按学习顺序逐步补齐。

仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文无 userinfo 测试尚待新增的说明为此前状态。

2026-10-22 无 userinfo 测试学习安排：用户问下一步。只读核对源码仍仅有 parse_sip_uri_with_userinfo 一个 URI 测试；独立只读复核认可下一小步只补 `parse_sip_uri_without_userinfo`。由用户调用 `parse_sip_uri("sip:example.com")`，用 assert_eq 完整比较手写 Ok(SipUri)：scheme 为 String::from("sip")、userinfo 为 None、host 为 String::from("example.com")，验证无 @ 分支。暂不同时展开后续 URI 组成或其他测试。

本轮仅讲解、核对覆盖并更新本卡与 CURRENT，测试执行 0 项，未运行编译、格式检查或完整工程检查；上一轮 14 项通过的结果仍为上一轮记录，不冒称本轮新验证。代理未改源码、测试、进度或需求，in_progress 及验收字段不变，未提交或推送。文档与差异校验单独核对，不作为协议行为验收。

2026-10-22 SIP URI 规则总结与首个测试复核：用户要求先总结 SIP URI 规则，并反馈已完成测试。本轮新增 `parse_sip_uri_with_userinfo`，直接调用 parse_sip_uri("sip:bob@example.com")，完整比较手写 scheme/userinfo/host；静态及独立只读复核认可断言。先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak -- --list` 列出 14 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak`，退出 0：14 passed、0 failed、0 ignored、0 filtered out，包含 1 项 URI、11 项起始行、2 项既有 CLI 单元测试。本轮 test target 无警告，`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；不外推为普通 binary 或严格 Clippy 已通过。

已核对 [RFC 3261 §19.1.1–19.1.4](https://www.rfc-editor.org/rfc/rfc3261.html#section-19.1.1)、[§25](https://www.rfc-editor.org/rfc/rfc3261.html#section-25) 与 [RFC 5630](https://www.rfc-editor.org/rfc/rfc5630.html)，讲解 scheme、可选 userinfo、必需 host、可选 port、分号 URI 参数与问号 headers；补充 IPv6 方括号、userinfo 内可合法含分号/问号、按组件处理百分号转义、大小写比较及 SIPS 逐跳 TLS 要求。说明组件边界必须先明确，不能全局按冒号/分号/问号拆分或先解码整条 URI；当前三字段模型仍是学习子集。

本轮按用户要求先总结规则，不安排或代写新实现。实际仅 1 项有 userinfo 的正常 URI 测试通过，无 userinfo、缺冒号及空字段等尚未实际测试；不将用户“测试完成”视为所有 URI 用例或整项 AC 已验收。总结后仍按原学习顺序由用户逐项补这些用例，再继续 URI 其他组成。仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试；未运行 Python/CLI 集成回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文 URI 尚无测试的说明为此前状态。

2026-10-22 URI 非空检查调整复核：用户已把 host 非空检查移到完整 if/else 之后、Ok(sipuri) 之前，两种分支共用；已有 userinfo 非空检查也保留。静态及独立只读复核确认本步三个空值输入均有错误路径：sip: 与 sip:bob@ 返回 host 错误，sip:@example.com 返回 userinfo 错误。尚无 URI 测试，以上仅为静态判断。

下一小步只由用户新增 `parses_sip_uri_with_userinfo`，调用 `parse_sip_uri("sip:bob@example.com")`，用 assert_eq 完整比较手写 Ok(SipUri)，期望 scheme 为 sip、userinfo 为 Some(String::from("bob"))、host 为 example.com。此后再逐项补无 userinfo 与边界测试，不将本步当成完整 SIP URI 支持。

`RUSTUP_AUTO_INSTALL=0 cargo check --locked --offline --all-targets`、`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 均退出 0，普通 binary 有 8 条、test target 有 2 条 dead_code 警告。本轮测试执行 0 项，未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络实验。仅同步记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试，未提交或推送。完整 URI 模型、完整字段校验、消息与序列化等仍未完成；下文 host 检查尚在 else 内的说明为此前状态。

2026-10-22 URI 非空检查作用域复核：用户已在 @ 分支正确拒绝空 userinfo，并新增 host 非空检查，但 host 检查仅在 else 内，含 @ 的分支仍漏检。静态及独立只读复核确认：sip:@example.com 与 sip: 已有错误路径，sip:bob@ 仍会返回空 host 的 Ok；以上为静态推导，未运行反例。下一小步只将现有 host 检查移到完整 if/else 结束后、Ok(sipuri) 之前，统一覆盖两个分支，再由用户补 URI 测试。

`RUSTUP_AUTO_INSTALL=0 cargo check --locked --offline --all-targets`、`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 均退出 0，普通 binary 有 8 条、test target 有 2 条 dead_code 警告。本轮测试执行 0 项，仍无 URI 测试；未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络实验。仅同步工作记录及本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试，未提交或推送；完整 URI、完整语法校验与序列化等仍未完成。

2026-10-22 URI 局部初始化调整复核：用户调整为先完整初始化局部 `let mut sipuri = SipUri { ... }`，在 if/else 中赋值并统一 Ok 返回。静态及独立只读复核确认初始化和作用域正确，当前结构可保留；但非空检查仍未补齐，`sip:`、`sip:bob@` 仍静态推导为空 host 的 Ok，`sip:@example.com` 仍为 Some(空字符串) 的 Ok。

下一小步只由用户补两处：在设置 Some(userinfo) 前检查 userinfo.is_empty() 并返回 Err；在统一 Ok 前检查 sipuri.host.is_empty() 并返回 Err。else 中重复的 sipuri.userinfo = None 可删除，因为已初始化为 None；不要求改成 match 风格。随后再补 URI 正常与边界测试，目前仍无调用 parse_sip_uri 的测试。

`RUSTUP_AUTO_INSTALL=0 cargo check --locked --offline --all-targets`、`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 均退出 0，普通 binary 有 8 条、test target 有 2 条 dead_code 警告。本轮测试执行 0 项，反例仅为静态推导；未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络实验。仅同步记录和本项备注并 render 更新视图，in_progress 与验收字段不变，代理未改源码或测试，未提交或推送。完整 URI 模型、完整语法校验与序列化等仍待继续。

2026-10-22 sipuri 作用域与初始化讲解：用户问如何将 sipuri 写在“全局”。结合当前 if/else 外声明的用法，先按函数内各分支共用局部变量解释：在分支外完整构造 `let mut sipuri = SipUri { ... }`，再修改字段；`let sipuri: SipUri;` 只声明未初始化，不能直接逐字段赋值。真正程序级全局使用 static，但每次解析的结果适合保留为局部值并返回，不引入共享可变状态。仅讲解片段，未代改源码或测试；测试执行 0 项，未运行编译或完整检查，不能沿用此前编译结果证明当前修改通过。下一步由用户完成初始化、非空检查及 URI 测试。

2026-10-22 SIP URI 解析初版评审：用户要求检查当前 URI 解析写法及优化。已阅读差异、相关类型与函数、所有调用搜索及 README，按单条离线学习输入评审。用户新增 `src/main.rs:92–113` 的 parse_sip_uri；静态及独立只读复核认为有、无 userinfo 的正常样例拆分和缺冒号返回 Err 符合思路，但尚未满足本步非空合同：`sip:`、`sip:bob@` 会返回空 host 的 Ok，`sip:@example.com` 会返回 Some(空字符串) 的 Ok。以上为静态推导，未创建或执行反例测试。

下一小步先由用户补空 host 与有 @ 时空 userinfo 的拒绝，再补 URI 正常/边界断言。可选简化是直接 match 或 if let 匹配 rest.split_once('@')，去掉 contains 的重复查找及其后不可能触发的缺 @ 错误分支；先得到借用字段，检查后统一构造一次 SipUri，避免两个分支重复构造。仅示意拆分片段，不代改源码。尚无任何 parse_sip_uri 的测试或调用，不把编译成功当作 URI 行为通过；完整 URI 校验等仍按原计划逐步完成。

`RUSTUP_AUTO_INSTALL=0 cargo check --locked --offline --all-targets`、`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 均退出 0。普通 binary 有 8 条 dead_code 警告，test target 有 SipUri 与 parse_sip_uri 尚未使用的 2 条警告。本轮测试执行 0 项，未重跑此前 11 项起始行测试，也未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验。

仅同步记录及本项备注并 render 更新视图，状态保持 in_progress、验收字段不变，代理未改源码或测试，未提交或推送。下文 URI 解析函数尚待新增的说明为此前状态。

2026-10-22 SIP URI 初步模型复核：用户新增 `src/main.rs:85–90` 的 SipUri，包含 scheme: String、userinfo: Option<String>、host: String，派生 Debug/PartialEq/Eq；静态及独立只读复核符合初步模型练习，尚无 parse_sip_uri 或 URI 测试。

下一小步只指导用户编写 `parse_sip_uri(input: &str) -> Result<SipUri, String>`，输入为不带 CRLF 的纯 URI 文本。先用 split_once(':') 拆第一个冒号，再按剩余部分是否含 @ 得到 Some(userinfo) 或 None，userinfo 保存 @ 前原文。手写期望：`sip:bob@example.com` 得到 sip / Some(bob) / example.com，`sip:example.com` 得到 sip / None / example.com；None 不等于 Some(空字符串)。缺冒号、空 host、有 @ 却空 userinfo 应返回 Err。此为简单样例解析练习，不宣称完整 SIP URI 校验；port、URI 参数、URI headers、转义等仍待逐步完成，既有 RequestLine.uri 继续保留原字符串。

`RUSTUP_AUTO_INSTALL=0 cargo check --locked --offline --all-targets`、`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 均退出 0。普通 binary 有 7 条 dead_code 警告，test target 有 SipUri 尚未构造的 1 条警告；不加 allow 隐藏，不声称严格 Clippy 通过。本轮测试执行 0 项，未重跑此前 11 项起始行测试，也未运行 Python/CLI 回归、完整 check.sh、release 构建、远端 CI 或网络/音频实验。

仅同步记录及本项备注并 render 更新视图，状态保持 in_progress、验收字段不变，代理未改源码或测试，未提交或推送。完整起始行语法、完整 URI 模型、完整消息及序列化仍未完成。下文 SipUri 尚待新增的说明为此前状态。

2026-10-22 统一入口响应错误传递复核：用户新增 `propagates_status_line_error`，调用 parse_start_line 并完整比较原错误；用户同步把数值转换错误文案改为 `status_code 无法转换为 u16`，源码与期望一致，符合本步合同。静态及独立只读复核认可；统一入口的请求/响应正常分流、混合大小写前缀及两种错误传递现有直接测试，不代表完整起始行语法或整个 AC 完成。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line -- --list` 列出 11 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0：11 passed、0 failed、0 ignored、2 filtered out（3 项请求行、3 项状态行、5 项入口）；2 项既有 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步按任务卡第 2 步进入 SIP URI，只由用户为 `sip:bob@example.com` 编写 SipUri 的初步三个字段：scheme: String、userinfo: Option<String>、host: String，派生 Debug/PartialEq/Eq。本步约定 userinfo 保存 @ 前的原文，样例为 Some("bob")；`sip:example.com` 对应 None。已核对 [RFC 3261 §19.1.1](https://www.rfc-editor.org/rfc/rfc3261.html#section-19.1.1) 的组成及 userinfo 可省略规则。此为逐步完善的数据模型，port、URI 参数、URI headers、转义及完整字段校验仍按原计划继续；不将三字段当成完整 URI 支持，既有 RequestLine.uri 暂仍保留原字符串。

本轮仅同步记录及本项备注并 render 更新视图，状态保持 in_progress、验收字段不变，代理未改源码或测试。未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送；完整消息及序列化仍未完成。下文响应错误测试尚待新增的说明为此前状态。

2026-10-22 统一入口请求错误传递复核：用户将 `propagates_request_line_error` 改为调用 parse_start_line，保留缺版本输入及完整 `Err(String::from("version 不存在"))` 断言；静态与独立只读复核确认，统一入口请求分支错误原样传递现已覆盖。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line -- --list` 列出 10 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0：10 passed、0 failed、0 ignored、2 filtered out（3 项请求行、3 项状态行、4 项入口）；2 项既有 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户补 `propagates_status_line_error`：调用 `parse_start_line("SIP/2.0 abc Busy Here\r\n")`，完整比较 `Err(String::from("code 无法转换为 u16"))`，确认响应分支原样传出数值转换错误。本轮仅同步记录及本项备注并 render 更新视图，状态保持 in_progress、验收字段不变，代理未改源码或测试。未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送；完整字段校验、完整消息及序列化仍未完成。下文仍调用专用请求解析器的说明为此前状态。

2026-10-22 请求错误测试调用路径复核：用户新增 `propagates_request_line_error`，完整比较 `Err(String::from("version 不存在"))` 的期望正确，但 `src/main.rs:212` 实际调用 parse_request_line，未经过统一入口。静态与独立只读复核确认本步错误传递合同尚未验证；下一小步仅由用户将该调用改成 parse_start_line，保留输入及完整断言，暂不推进响应错误测试。

先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line -- --list` 列出 10 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0：10 passed、0 failed、0 ignored、2 filtered out（4 项直接请求行、3 项状态行、3 项入口）；2 项既有 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。测试全绿不代表未经过的入口错误路径已覆盖。

本轮仅同步工作记录及本项备注并 render 更新视图，状态保持 in_progress、验收字段不变，代理未改源码或测试。未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。完整字段校验、完整消息及序列化仍未完成；下文请求错误测试尚待新增的说明为此前状态。

2026-10-22 混合大小写入口测试复核：用户新增 `parse_start_line_with_mixed_case_response_prefix`，直接调用统一入口并完整比较 Response 及三个字段，version 保留原始 `sIp/2.0`。先执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line -- --list` 列出 9 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0：9 passed、0 failed、0 ignored、2 filtered out（3 请求行、3 状态行、3 入口）；2 项既有 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 无警告。

下一小步只由用户补 `propagates_request_line_error`：调用 parse_start_line，输入 `INVITE sip:bob@example.com\r\n`（缺少版本），完整比较 `Err(String::from("version 不存在"))`，确认请求解析错误原样传出；响应错误路径随后再补。本轮仅同步工作记录及本项备注并 render 更新视图，状态保持 in_progress、验收字段不变，代理未改源码或测试。未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或网络/音频实验，未提交或推送。完整字段校验、完整消息及序列化仍未完成；下文混合大小写测试尚待新增的说明为此前状态。

2026-10-22 统一入口请求/响应测试复核：用户新增 `src/main.rs:169–194` 的 parse_start_line_as_request、parse_start_line_as_response，直接调用统一入口并完整比较手写 Request/Response 变体与内部字段。静态与独立只读复核认可断言；先用 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line -- --list` 列出 8 项，再运行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0，实际 8 passed、0 failed、0 ignored、2 filtered out。包含 3 请求行、3 状态行和 2 入口测试；2 项既有 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0，本轮 test target 没有警告，不外推为普通 binary 或严格 Clippy 已通过。

下一小步由用户补 parses_start_line_with_mixed_case_response_prefix，输入 `sIp/2.0 486 Busy Here\r\n`，期望 Response，version 仍原样为 `sIp/2.0`；eq_ignore_ascii_case 只比较，不改输入。随后再逐步补错误传递。当前仅上述 8 项通过，不代表完整字段、消息或序列化完成。本轮只同步记录和本项备注、由 render 更新视图，in_progress 与验收字段保持，用户源码与测试未改；未运行 Python/CLI 回归、Clippy、完整检查、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文入口尚未有测试或 test target 尚有未使用警告的说明为此前状态。

2026-10-22 统一起始行入口静态评审：用户完成 `src/main.rs:148–162` 的 parse_start_line，使用 get(..4) 与 eq_ignore_ascii_case 判断行首 SIP/，响应/请求分支各自调用一个既有解析函数，用问号传递原错误，成功值包装对应 StartLine 变体。静态与独立只读复核均认为本步整合正确；尚无直接调用统一入口的测试，不把原解析器测试当作入口已验证。

实际先列出 6 项，再执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0，6 passed、0 failed、0 ignored、2 filtered out，覆盖既有 3 请求行和 3 状态行用例；2 项 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；test target 报告 StartLine、parse_start_line 尚未使用的两条 dead_code 警告。下一小步由用户补 parses_start_line_as_request、parses_start_line_as_response，均直接调用统一入口，完整比较手写的 Request/Response 枚举期望，随后再逐步补大小写与错误传递测试。本轮仅同步记录与本项备注、由 render 更新视图，in_progress 与验收字段保持，用户源码与测试未改；未运行 Python/CLI 回归、Clippy、完整检查、release 构建、远端 CI 或网络/音频实验，未提交或推送。下文统一入口尚未整合等说明为此前状态。

2026-10-22 枚举包装讲解：用户问响应分支成功后如何包装为 StartLine::Response。按三个类型步骤说明：parse_status_line(input)? 取得 StatusLine，StartLine::Response(status_line) 构造 StartLine，再用 Ok 包装为 Result<StartLine, String>；问号遇 Err 则从统一入口直接返回原错误。仅展示响应分支示例，具体分支整合仍由用户编写。本轮只同步记录，未改源码或测试，测试执行 0 项，未运行编译或完整检查；下一步由用户完成两个分支及对应包装。

2026-10-22 input 范围澄清：用户把 input 理解为同时包含请求和响应的文本，之前讲解未明确本函数一次只接收一条含 CRLF 的起始行。澄清请求与响应是不同消息；各有一条起始行时，parse_start_line 确实分别调用一次、合计两次，每次只走一个分支。若原始文本拼接了多条消息，需先在外层识别消息边界并提取各起始行；本函数不承担消息流拆分，也不把该外层工作纳入本步实现。只同步记录，未改源码或测试，测试执行 0 项，未运行编译或完整检查；下一步仍由用户完成单条起始行入口。

2026-10-22 调用层次讲解：用户问 parse_start_line 是否解析两次。说明计划结构中，调用者进入统一入口后，由入口检查前缀并只调用选中的一个专用解析函数；函数调用有两层，字段拆分解析只执行一次，if/else 不会同时运行两个分支。当前源码尚未接入专用解析调用，以上是待用户完成的结构。本轮只讲解与同步记录，未改源码或测试，测试执行 0 项，未运行编译或完整检查；下一步仍由用户完成分支、包装和错误传递。

2026-10-22 布尔分支补充讲解：用户问 is_response 如何判断分流。说明该变量只保存前缀比较得到的 bool，`if is_response` 在 true 时进入响应分支，else 在 false 时进入请求分支；`!` 表示取反。仅给出带说明注释的分支示意，具体解析、变体包装与错误传递仍由用户编写。本轮只同步记录，未改源码或测试，测试执行 0 项，未运行编译或完整检查；前缀判断仍不等于完整输入校验。

2026-10-22 起始行分流讲解：用户问如何区分请求与响应。结合当前源码说明：响应以版本标记 `SIP/` 开头，请求以方法名开头、版本在末尾；当前 `!is_response` 就返回错误会误拒绝正常请求，应进入请求行解析分支，响应分支调用状态行解析，再分别包装 StartLine 变体。前缀仅选择解析器，不能证明输入合法，错误由所选解析器返回，现有字段校验仍不完整。已核对 RFC 3261 §7.1/§7.2；只讲解与同步记录，未改源码或测试，测试执行 0 项，未运行编译或完整检查。下一步由用户完成两个分支。

2026-10-22 正则前缀判断补充：正则可以表达此判断；使用 regex crate 时，`(?i-u)^SIP/` 限定字符串开头并作 ASCII 大小写无关匹配。已核对其官方文档；当前 Cargo.toml 无依赖，建议此固定前缀仍用标准库，若采用正则应编译后复用。本轮仅讲解与同步记录，未添加依赖或改源码，测试执行 0 项，未运行编译或完整检查；下一步仍由用户完成统一入口。

2026-10-22 行首大小写判断讲解：用户询问如何忽略大小写判断以 `SIP/` 开头。建议 `input.get(..4).is_some_and(|prefix| prefix.eq_ignore_ascii_case("SIP/"))`：安全取得前 4 字节，仅对前缀作 ASCII 大小写无关比较，短输入或非字符边界返回 false，不分配新字符串。本轮仅讲解并同步 CURRENT 与任务卡，不改源码、测试、需求或进度；测试执行 0 项，未运行编译或完整检查。下一步仍由用户编写统一起始行入口；此前通过的测试不代表当前空函数已通过验证。

2026-10-22 合法空原因短语测试复核：用户说“通过了”，实际新增 `src/main.rs:181–192` 的 parses_status_line_with_empty_reason_phrase，输入 `SIP/2.0 200 \r\n` 保留状态码后的 SP，完整比较手写 StatusLine，reason_phrase 为空。该“通过”只对应本测试，不是 AC-01-01 整项验收。先用 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line -- --list` 列出 6 项，再执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0，实际 6 passed、0 failed、0 ignored、2 filtered out；含 3 请求行、3 状态行测试，既有 2 项 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；test target 有 StartLine 尚未使用的一条 dead_code 警告。

下一小步由用户编写 `parse_start_line(input: &str) -> Result<StartLine, String>`：输入仍含 CRLF，按行首 SIP/ 版本标记（大小写不敏感）分流，复用已有两个解析函数，分别包装 Response/Request 变体，错误直接传递；不重复实现拆字段，也不以全文 contains 作分流。独立只读复核认可测试与本步合同。仅同步记录和本项备注、由 render 更新视图，状态仍 in_progress，验收字段不变，用户源码和测试未改。字段内部完整校验、完整消息与序列化等尚未完成；未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络/音频实验，未提交或推送。以下空原因短语测试尚待新增的说明为此前状态。

2026-10-22 状态行 HTAB 测试复核：用户新增 `src/main.rs:167–178` 的 parses_status_line_with_tab_in_reason_phrase，完整比较手写 StatusLine，确认 reason_phrase 原样保留 `Busy\tHere`。先用 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line -- --list` 列出 5 项，再执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0，实际 5 passed、0 failed、0 ignored、2 filtered out；含 3 请求行和 2 状态行，既有 2 项 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；test target 仍有 StartLine 尚未使用的一条 dead_code 警告。该修正现已有实际测试覆盖，不再停留于静态判断。

下一小步由用户补 `parses_status_line_with_empty_reason_phrase`，输入 `SIP/2.0 200 \r\n`，保留 200 后的分隔 SP，手写期望 reason_phrase 为空字符串。此合法边界核对后再进入统一 parse_start_line。独立只读复核认为当前 Tab 断言满足练习，不改源码或测试；仅同步记录和本项备注、由 render 更新视图，状态仍 in_progress、验收字段保持。完整字段校验、消息和序列化等仍未完成；未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络/音频实验，未提交或推送。以下 Tab 测试尚待编写的说明为此前状态。

2026-10-22 状态行 HTAB 范围调整复查：用户已将整行检查收窄为 CR/LF，并分别在 version、code 上拒绝 HTAB，原因短语保留未过滤的剩余片段。静态检查与独立只读复核确认符合本步字段分工；源码中尚未新增原因短语含 Tab 的测试，不宣称该输入已实际验证。下一小步仍由用户新增 `parses_status_line_with_tab_in_reason_phrase`，输入 `SIP/2.0 486 Busy\tHere\r\n`，手写期望 reason_phrase 为 `Busy\tHere`。

本轮先列出 4 项，再执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line`，退出 0，实际 4 passed、0 failed、0 ignored、2 filtered out；既有 CLI 单元测试未运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；test target 有 StartLine 尚未使用的一条 dead_code 警告。仅同步记录和本项备注、由 render 更新视图；in_progress 与验收字段保持，用户源码和测试未改。未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络/音频实验，未提交或推送。下文整行 HTAB 拒绝尚待修改的记录为此前状态。

2026-10-22 状态行初版评审：用户已在 `src/main.rs:112–140` 编写 parse_status_line，并新增正常 `SIP/2.0 486 Busy Here\r\n` 的完整 StatusLine 断言。当前未 filter/trim 的 split + Vec + join 能保留原因短语及其中连续空格；正常样例通过。先用 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak line -- --list` 列出 4 项，再执行相同命令去掉 `-- --list`，实际 4 passed、0 failed、0 ignored、2 filtered out（既有 CLI 单元测试未运行），退出 0。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；test target 有 StartLine 尚未使用的一条 dead_code 警告。

下一小步只修正原因短语的 HTAB 处理：RFC 3261 §25 允许 Reason-Phrase 包含 HTAB，但当前 `:118` 对整行拒绝 Tab，会静态推导为误拒绝 `SIP/2.0 486 Busy\tHere\r\n`；本轮未执行此反例。由用户将 HTAB 限制按字段处理，保留整行 CR/LF 拒绝，允许原因短语保留 Tab，并补 `parses_status_line_with_tab_in_reason_phrase` 的手写期望成功测试。独立只读复核同意；版本、三位状态码及其范围等校验继续后续小步，不把当前 4 项通过外推为整项验收。RFC 4475 §3.1.1.13 也允许宽容接收缺空原因前 SP，不将缺 SP 必须拒绝新增为门槛。本轮只更新记录与本项备注，并 render 更新视图；in_progress 与验收字段保持，用户源码和测试未改。未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络/音频实验，未提交或推送。

2026-10-22 split/join 补充讲解：用户提出先取两段、再将剩余段用空格 join。此思路可行；`str::split` 返回 Split 迭代器，标准库未提供迭代器的 join，可先 `collect::<Vec<_>>()` 再调用切片的 `join(" ")`。未过滤、未 trim 的 `split(' ')` 保留空段，再以相同分隔符 join 可保留剩余短语中的连续空格；不能误说 join 必然折叠空格。代价是临时 Vec 和新 String，且剩余零段与单个空段 join 后都为空字符串，需另行区分缺分隔符与合法空原因短语。下一步仍由用户选用该方案或 splitn 并完成正常样例。仅同步工作记录，不改源码、测试或进度；项目测试 0 项，未运行编译或完整检查。

2026-10-22 状态行解析思路讲解：用户询问 `src/main.rs:112–126` 如何继续，仅授权讲解。建议先用 `splitn(3, ' ')` 学习标准单 SP 输入的拆分：版本、状态码文本、完整原因短语；数字用 `parse::<u16>()` 配合 `map_err` 转换，再构造 StatusLine。原因短语不可统一过滤或 trim；RFC 3261 §25 允许其为空及含 SP/HTAB，因此当前复制自请求行的整行 HTAB 拒绝不能直接当作完整状态行规则。字段间多 SP 容错及版本、状态码完整校验后续由用户逐步补齐，不改变既有请求行容错策略。下一步由用户先完成正常 `SIP/2.0 486 Busy Here\r\n` 样例，再手写期望测试。本轮只更新 CURRENT 与任务卡，未改源码、测试或进度；项目测试执行 0 项，未运行编译、格式、完整工程检查或网络实验，不将示例视为已验证实现。

2026-10-21 请求行三个单元测试评审：用户已亲自新增单空格、多空格容错和缺字段三项测试，前两项完整比较手写 RequestLine，缺字段项检查错误结果。先用 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak request_line -- --list` 核对选中 3 项，再执行 `RUSTUP_AUTO_INSTALL=0 cargo test --locked --offline --bin yakyak request_line`，退出 0，实际 3 passed、0 failed、0 ignored、2 filtered out；过滤掉的是既有 CLI 单元测试，不计为已运行。`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 退出 0；test target 仍报告 StatusLine、StartLine 两条 dead_code 警告。第三个函数名把 fields 拼成 fileds，是不阻塞当前断言的小拼写问题，由用户自行修正，代理未改源码。

下一小步由用户编写 `parse_status_line(input: &str) -> Result<StatusLine, String>`，先解析含 CRLF 的 `SIP/2.0 486 Busy Here`：保存版本、将状态码解析为 u16、完整保留原因短语；原因短语可含空格，不能沿用对所有片段统一 filter 的办法。当前仅三项请求行测试通过，不代表字段内部语法、完整消息、序列化、独立验证或本 AC 整项完成。本轮仅更新工作记录与本项备注、由 render 更新视图；in_progress 与验收字段保持，源码和测试由用户编写并保留原样，未运行 Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络/音频实验，未提交或推送。下文三项测试尚待编写等记录属于此前状态。

2026-10-21 空格策略纠正（以此为准）：用户指向 `src/main.rs:97`，当前确实为 `line.split(" ").filter(|v| !v.is_empty())`。筛选后不存在空字段；对 `"INVITE  SIP/2.0\r\n"` 只剩两段，应走缺第三字段的 Err，不能再沿用未过滤版本的空 uri 反例。补查 [RFC 4475 §3.1.2.9](https://www.rfc-editor.org/rfc/rfc4475.html#section-3.1.2.9) 并独立复核：接收端可拒绝额外 SP，也可忽略字段间额外 SP；发送或转发时需去掉额外 SP。此前把严格拒绝说成唯一正确做法过于绝对，现撤回这项附加门槛；下文相关记录保留为已被纠正的历史说明。保留用户的容错写法，无需再加重复非空检查；不据此宣称所有语法校验完成或接受任意畸形输入。

下一小步由用户编写正常单空格、多个空格且三个字段齐全、缺字段三组单元测试，分别期望正确 RequestLine、相同 RequestLine、Err；期望手写，不由解析器生成。后续序列化输出标准单空格。当前 cargo check --locked --all-targets --offline 与 cargo fmt --all -- --check（均带 RUSTUP_AUTO_INSTALL=0）退出 0，有四条 dead_code 警告；项目测试执行 0 项，未运行 Clippy、完整检查、release 构建、远端 CI 或协议/网络/音频实验。仅纠正任务卡、CURRENT 与本项进度备注，并 render 更新视图；状态仍 in_progress，验收字段不变，用户源码与测试未改，未提交或推送。此为纠正代理对规范的误读，不修改目标真源。

2026-10-21 过滤结果与原始格式的区别：用户指出 split 后 filter 非空段，后续字段应不会为空。此判断正确；需要保留的校验是原始字段间分隔格式，直接删空段会把 `"INVITE  sip:bob@example.com SIP/2.0"` 的四段变为三个非空段，使后续缺字段和额外字段检查均通过。按已有单空格请求行约定，应保留 split 结果并拒绝空段，而非删除空段。仅更新讲解记录，未改源码、测试或进度；项目测试执行 0 项，未运行项目编译、格式、完整检查或网络实验。

2026-10-21 非空检查位置讲解：用户建议放在 parts 声明处集中处理。集中校验可行，但若对 split 迭代器直接 filter 非空段，会跳过空字段，把连续空格等格式错误掩盖；建议保留 split 的原始结果，读取三个字段后，用一次 method/uri/version 的 is_empty 条件返回 Err，已有缺字段和第四字段检查保留。三处 Option::filter 是另一种写法，不是必须。下一步由用户补上统一校验及对应测试。本轮仅更新学习记录，未改源码、测试或进度；项目测试执行 0 项，未运行项目编译、格式、完整检查或网络实验。

2026-10-21 整行与字段非空的区别：用户询问生成 line 时已有 filter，为何字段仍可能为空。本轮运行一次独立 Rust 标准库演示，`Some("INVITE  SIP/2.0").filter(|v| !v.is_empty())` 仍保留整行，split 后得到 `["INVITE", "", "SIP/2.0"]`；示例编译、运行均退出 0。这只演示 filter/split，不调用项目解析器，不计为协议测试。下一步仍由用户为三个字段分别加非空检查。仅更新讲解记录，不改项目源码、测试或进度；项目测试执行 0 项，未重跑项目编译、格式、完整检查或网络实验。

2026-10-21 请求行空字段复查：用户询问“做好了，下一步做什么”。当前 `src/main.rs:93–95` 已拒绝移除末尾 CRLF 后剩余的 CR/LF/HTAB，但 `:90` 的 filter 只检查整行非空；`:98–100` 三个字段仍只拒绝 None。静态反例 `"INVITE  SIP/2.0\r\n"` 会产生空 uri 并返回 Ok，未实际执行此反例。下一小步由用户分别在三个 parts.next() 后添加非空 filter，再给该反例编写应返回 Err 的单元测试；先完成这处边界再继续状态行解析。cargo check --locked --all-targets --offline 与 cargo fmt --all -- --check（均带 RUSTUP_AUTO_INSTALL=0）退出 0，存在四条未使用代码警告（各 target 重复报告）。本轮仅更新学习评审记录，源码、测试、需求与进度未改，测试执行 0 项；未运行 Rust/Python/CLI 回归、Clippy、完整检查、release 构建、CI 或网络实验，未提交或推送。

2026-10-21 字段内字符检查讲解：建议在 strip_suffix 移除合法末尾 CRLF 后、split 前，检查 line 是否含 CR、LF 或 HTAB，存在即 Err；按 RFC 3261 §7.1 核对，此检查不替代方法、版本和 URI 语法校验。当前 filter 位于 strip_suffix 后，仅检查整行非空，仍需对三个 parts.next() 的结果分别检查非空。下一步由用户亲自补充字符与空字段检查。本轮只更新工作记录，未改源码、测试或进度；测试执行 0 项，未运行编译、格式、Clippy、完整检查或网络实验。

2026-10-21 请求行解析初版评审：用户已在 src/main.rs:85–105 编写 strip_suffix、split、三个字段读取、额外字段拒绝和结构体构造。按离线单条输入练习静态核对，缺字段、多余字段和缺末尾 CRLF 已有错误路径；主要问题是 Some(空字符串) 不会被 ok_or_else 拒绝，例如 INVITE 后连续两个空格可产生空 URI 并返回 Ok。字段内 CR/LF/HTAB 及方法、版本、URI 语法尚未校验，不能当作完整 SIP 解析；建议下一小步先为三个字段补非空检查，再由用户编写正常、缺字段、空字段与多字段测试。RUSTUP_AUTO_INSTALL=0 cargo check --locked --all-targets --offline 与 RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check 均退出 0；前者有 RequestLine、StatusLine、StartLine、parse_request_line 四条未使用代码警告（两个 target 重复报告）。测试执行 0 项；未运行 Rust/Python/CLI 回归、Clippy、完整 check.sh、构建产物验收或网络实验。源码调用搜索仅找到函数定义；src 下 AGENTS.md 搜索无匹配退出 1，非测试失败。代理只更新工作记录，未改源码、测试、需求或进度，未提交或推送。

2026-10-21 请求行解析讲解：用户询问 src/main.rs:86 的样例如何解析，当前函数仍为 todo!()。本轮只讲解先用 strip_suffix 检查并移除末尾 CRLF、按单个空格 split、检查恰好三个非空字段，再构造 RequestLine；不用 trim 或 split_whitespace 掩盖格式错误。核对 RFC 3261 §7.1；字段内部语法与 URI 解析仍需后续学习，不将拆分视为完整协议校验。下一步由用户先编写 CRLF 检查，再继续拆字段。代理未改源码或测试，未改进度；测试执行 0 项，未运行编译、Rust/Python/CLI 测试、完整检查或网络实验；未提交或推送。

阶段：第 00 章五项目标 AC-00-01～05 均已人工验收通过；INIT-001 也已单独通过。通信核心尚未实现。
当前任务：[AC-01-01：消息与 URI 模型](tasks/AC-01-01.md)，in_progress。用户已亲自编写两种起始行解析及统一入口，3 项请求行、3 项状态行和 2 项入口测试已通过；下一步补入口混合大小写前缀用例，完整字段校验、错误传递测试、序列化与其他协议测试尚未完成。

当前学习位置（2026-10-21）：用户说“enum StartLine 写好了”，实际在 `src/main.rs:80–83` 定义 Request(RequestLine)、Response(StatusLine)，并给三种类型添加 Debug、PartialEq、Eq，符合当前练习。代理只读评审；`RUSTUP_AUTO_INSTALL=0 cargo check --locked --all-targets --offline`、`RUSTUP_AUTO_INSTALL=0 cargo fmt --all -- --check` 均退出 0。当前有 RequestLine、StatusLine、StartLine 的三条 dead_code 警告（各 target 重复报告），不冒称严格 Clippy 或完整 CI 通过，不用 allow 隐藏。下一小步只指导用户写请求行解析：输入含末尾 CRLF，成功返回 RequestLine，格式错误返回 Err；先核对正常样例与缺字段样例。代理仍只讲解和评审，不改用户源码。

本轮仅同步学习评审记录和本项进度备注，并通过 render 更新视图；状态保持 in_progress，提交、证据与整项人工审阅字段仍为空。测试执行 0 项；未运行 Rust/Python/CLI 回归、Clippy、完整 check.sh、release 构建、远端 CI 或协议/网络/音频实验。用户未提交的 `src/main.rs` 保留原样；本轮不提交或推送。下文准备阶段的 not_started 与仅两份文档变化等记录属于当时状态。

此前第 1 步学习安排（2026-10-21）：用户要求“一个一个来，先开始1”。按此前用户亲自编写核心的安排，先讲请求行、状态行及各自字段，再由用户设计 Rust 类型，代理负责讲解和评审。当轮只更新学习位置，未修改源码、创建协议测试或推进验收；测试执行 0 项。

2026-10-21（Asia/Shanghai），用户要求“准备 AC-01-01”。本轮已从干净的本地 main `d048f217f2ff2e79fef5ae57c9da5ed404064b10` 创建 `docs/ch01-ac01-01`，整理学习顺序、正常样例与单元/属性设计，划清 AC-01-02～04 的验收边界。仅新增任务卡并更新本入口；不改变需求、进度、既有验收或 ADR，不实现通信核心。本轮未 fetch，未核对实时远端；未提交、推送或创建 PR。

原 CURRENT 已逐字节备份到忽略目录 `artifacts/ac01-01/20261021-preparation-xwzoqmof/`。准备检查的 `validate --git`、`render --check`、`git diff --check` 均退出 0，27 章/125 项有效、30 个生成视图一致。范围/链接复查退出 0，33 个本地链接存在，仅两份准备文档变化，index 为空，需求与进度未变；独立只读审阅无实质问题。首轮链接核对退出 1，因任务卡误引用不存在的验收入口，已改为现有进度表并复查通过，记录留在上述目录。本轮测试执行 0 项；未运行 Python/Rust/CLI 回归、构建、CI 或协议/网络/音频实验。

下一步由用户为 parse_start_line 补混合大小写响应前缀测试，代理讲解、评审与建议测试；本轮不构成代理代写实现授权。LOCAL-001 按原决定保留 pending，不阻塞离线模型学习，解除条件不变。

## 历史记录（以下保留当时状态与授权）

此前当前任务：[AC-00-05](tasks/AC-00-05.md) passed。2026-10-21，用户收到[正式验收报告](../acceptance/ch00/AC-00-05.md)、报告提交 D 和整项审阅提示后回复“通过”，已登记人工整项验收。reviewer 为“用户（本会话明确确认）”，reviewed_at 为 2026-10-21。

ADR Git 交付授权（2026-10-21）：用户在推送 `docs/confirm-adr-0001-0003`、创建 PR、等 CI 后普通合并、同步本地 main 的四步说明后回复“操作一下”。执行前分支 clean，HEAD 为 `e98a823b29f5d5b086f821ee60ce9ab8904472e1`，实际远端 main 为 `ae7391192e89b78c3edaf364d62af48358c1ef0e`，远端尚无本分支或 PR。先保存本轮三份工作记录，再推送最终 head；最终 PR、CI、合并与本地/远端 main、clean 核对以忽略目录 `artifacts/adr-review/20261021-delivery-adr-0001-0003-d6smxomo/final-closeout.json` 为准，不预写成功或为回填编号反复追加提交。

用户另说明旧凭据内容已遗忘，本轮选择暂缓核实；LOCAL-001 保留 pending，不阻塞本次文档交付和后续离线模型学习安排。此说明不证明旧凭据已失效或撤销；保留原解除条件，不读取私有备份内容。后续 AC 仍未开始，下一步可另行准备 AC-01-01。

本轮交付记录的 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项有效、30 个生成视图一致。范围/链接核对退出 0，仅三份工作记录变化，完整分支仅六份文档变化，43 个本地链接目标存在；独立只读审阅无实质问题。本地测试执行 0 项，未重复 Python/Rust/CLI 回归或构建；最终 head 与合并后 main 的完整检查由各自实际双平台 CI 单独核对。协议/网络/音频实验未运行。

最新补充（2026-10-21）：用户在讨论工程组织与证据制度后回复“同意01 和03”，已单独登记 [ADR-0001](../architecture/adr/0001-small-core.md) 和 [ADR-0003](../architecture/adr/0003-documentation-evidence.md) 为 Accepted，并将 REVIEW-001 记为 resolved。ADR-0001 明确采用单 Cargo 包起步、内部按职责分模块，稳定后按实际需求拆包；同包以后可按需包含 library 和 binary 两个 crate。问题清单现为 7 resolved、1 pending，LOCAL-001 凭据处理仍待用户核实。

本次只登记设计确认，AC-00-05 的固定提交、报告、passed 与既有证据保持不变，后续 AC 仍未开始。登记分支为 `docs/confirm-adr-0001-0003`；修改前六份文档已逐字节备份，确认原话、检查和本地提交回执放在忽略的 `artifacts/adr-review/20261021-confirm-0001-0003-m7ecbtc9/`。下文 ADR 尚 Proposed、两项 pending 等说明属于此前各轮的历史状态。

ADR 登记检查（历史）：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项结构有效、30 个生成视图一致。一次性范围/链接检查确认仅六份文档变化、43 个本地链接目标存在、LOCAL-001 原行不变；独立只读审阅无须修正的实质问题。测试执行 0 项，未运行 Python/Rust/CLI 回归、构建、CI 或协议/音频实验；当时仅保存本地文档提交，未推送，现按上述授权交付。

本轮 Git 收尾授权（2026-10-21）：用户要求“把05剩余的事情处理一下”，据此交付本分支、创建 PR、等待最终 head 的 Ubuntu/macOS CI，全部成功后普通 merge，核对主线 CI 并同步本地 main。执行前实际 origin/main 为准备基线，远端无本分支或 PR，工作区 clean；独立代理只读审阅完整文档差异，无实质问题。后续 AC 不因本轮交付自动开始。

交付分支：`docs/ch00-ac00-05`，工作目录为用户原项目目录；准备基线 `8720de6c2ad8b62f6ad491497984a25bc2d42385`。固定 C 为 `275b2eedd4a33b0fec28bc278e9d01a56e9f657d`，第 4 项补录为 `a65a545ee4cbc0c49d7ba670c4068c0af0dcc937`，报告 D 为 `e65d84bfccb08e83511c583f23ba3d8c422ddc6c`，人工登记为 `b195be459a1b1917fa5a9f26445b849447b7f93c`。先保存本轮交付说明，再一次推送并创建 PR；最终 head、PR、CI、普通合并、本地/远端 main 和 clean 核对以收尾回执为准，不在本提交内自引用或预写远端成功。

交付预检已在全新本地 clone 明确检出人工登记提交 `b195be459a1b1917fa5a9f26445b849447b7f93c`：doctor、完整 `bash scripts/check.sh` 均退出 0，前后 clean；21 项 Python、2 项 Rust 单元、6 项 CLI 测试及 fmt/Clippy/release 构建通过。此为后续 Git 交付预检，与原始文档 C 的复验范围分开，不证明协议/音频功能。最终交付 head 和合并后 main 的实际双平台 CI 另行核对。

本轮 2 份工作记录已备份，预检、PR 正文、远端操作、CI 和最终收尾回执归档于忽略的 `artifacts/ch00/275b2eedd4a33b0fec28bc278e9d01a56e9f657d/git-closeout-cd97wp2r/`；最终状态读取其中 `final-closeout.json`，不为补 PR 编号反复追加已交付分支。保留历史分支、tag、已有工作树、C/D/人工登记与原始证据；不部署、发布、删除历史或实现下一 AC。

交付记录的 validate --git、render --check、git diff --check 均退出 0，27 章/125 项有效、30 个生成视图一致；未为两份记录重复完整预检，最终 head 与 main 的完整检查以各自实际 CI 为准。

此前人工登记（历史）只更新本项的报告、状态、生成视图和工作记录；被测 C、原始证据与未运行限制不变，ADR-0001/0003 仍 Proposed，REVIEW-001 和 LOCAL-001 保留 pending。6 份记录已备份并核对，授权、检查与最终提交回执在忽略的 `artifacts/ch00/275b2eedd4a33b0fec28bc278e9d01a56e9f657d/human-acceptance-gkgsoy0i/`。当时未执行远端交付，现按本轮 Git 收尾授权处理；仍不开始下一 AC。

本轮人工登记的 validate --git、render --check、git diff --check 及范围/链接核对均退出 0，27 章/125 项有效、30 个生成视图一致、43 个本地链接存在；仅本项状态与人工审阅改变，汇总为 5 passed、120 not_started。独立代理只读核对无实质问题；测试执行 0 项，未重跑 Python/Rust/CLI、构建、CI 或协议/音频实验。结果保存为本地人工登记提交，最终 SHA 与 clean 回执另存上述目录。

第 5 项报告登记（历史）采用既有验收模板，逐项对应原目标、状态与历史初始记录、不支持表、问题清单、固定 C 的实际命令/数量/结果及限制。当时仅将本项改为 ready_for_review，整项审阅字段留空；本次已按上述明确确认另行登记 passed。备份与报告登记检查在忽略的 `artifacts/ch00/275b2eedd4a33b0fec28bc278e9d01a56e9f657d/report-registration-5mfccaco/`，不混入 C 的原始结果。

第 5 项登记检查：validate --git、render --check、本轮 21 项 Python 回归、git diff --check 和范围核对全部退出 0，27 章/125 项有效、30 个生成视图一致，21 项通过且无失败/错误/跳过，43 个本地链接存在。独立代理只读评审未发现实质问题；仅 6 份登记文档变化，原始 C、清单和其他 AC 不变。检查来自未提交登记工作区，和 C 的干净结果分开，不冒称对 D 做过独立复验。完整记录保存为本地 D，最终 SHA 与 clean 回执另存上述目录；未运行 Rust/CLI、构建、CI 或协议/音频实验，未推送，待用户整项审阅。

第 4 项实际结果：`git clone --no-local --no-checkout` 后检出完整 C；检查前后 clone 的 HEAD 均为 C、status 均空，执行完原工作区也仍 clean。macOS 27.0.1 arm64、Python 3.13.8、Git 2.50.1 上，validate --git、render --check、21 项 unittest、git diff --check 和固定 C 的 git show --check 全部退出 0。27 章/125 项有效、30 份生成视图一致，21 项通过、0 失败/错误/跳过。

原始证据在忽略的 `artifacts/ch00/275b2eedd4a33b0fec28bc278e9d01a56e9f657d/documentation-review-5bh228z4/`；日志、环境、摘要和 32 份文件哈希齐全，独立代理逐项复核 32/32 一致。SHA256.json 自身哈希为 `4e6964c73f015b91d70f845b3aa43e6b06db45077c7ff0472a21969592f0c5c9`。Rust/CLI、构建、远端 CI 与协议/抓包/音频实验未运行；文档检查不证明通信功能。实际结果在 C 之后补录为本地记录提交，不改 C 或原始证据。

第 4 项补录的 validate --git、render --check、差异与范围核对全部退出 0；仅本卡对应任务文档、CURRENT、progress.json、生成 PROGRESS 共 4 份记录变化，27 个本地链接存在。原始证据清单不变，独立 clone 仍为完整 C、clean；未重复运行回归或创建第 5 项报告。补录日志与独立复核摘要在上述证据目录 `registration/`。

第 2 项已新增[不支持功能表](../UNSUPPORTED.md)，在根 README 和文档地图添加入口，依据已确认的 SCOPE v1.0 记录“范围内尚未实现”和“范围排除”，并列出对应章节、验证边界及更新方式。普通录音、PCM、DTMF 等保留在范围内；测试端准备不计为 YakYak 通信支持。

第 3 项核对：初始提交 `f014ea61de4aea872742e266ce398a8b06d65f5f` 的 125 项全部 not_started，提交/证据/审阅字段为空；前四项固定提交均真实且为当前 HEAD 的祖先，进度、五份报告和既有人工确认一致。独立代理核对相应本地证据引用存在，AC-00-03 清单 73/73、AC-00-04 清单 42/42 文件均存在；未重算原始哈希或重跑历史实验。问题清单过期描述已修正，8 条记录保留 6 resolved、2 pending；待确认 ADR 和凭据处理的解除条件保留，详见任务卡与 [KNOWN_ISSUES](KNOWN_ISSUES.md)。

第三项通过与整项人工验收分开记录：第 4 项完成时本项为 in_progress，第 5 项报告先登记 ready_for_review；本次收到用户整项确认后才记 passed。SCOPE、targets.json、AC-00-01～04 的通过状态和 ADR 状态不变，后续 AC 仍未开始。

第 4 项执行前（历史）：本项 8 份准备文档已完整备份并核对，index 为空、无其他改动；备份位于忽略的 `artifacts/ch00/ac00-05/20261021-stage4-preparation-oo4e4di0/`。独立代理只读预审未发现必须修正的实质问题。当时先创建 C，再以完整 C 新建本地独立 clone，实际结果现已在 C 之后登记。

第三项登记检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 及一次性链接/范围核对均退出 0，27 章/125 项有效、30 个生成视图一致，3 份工作文档的 34 个本地链接存在。测试执行 0 项，未运行回归、构建、CI、网络或音频实验。历史证据只做记录与元数据核对，不冒称重新测试；命令与一次无匹配搜索说明见任务卡。

第二项检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 及一次性链接/范围检查均退出 0：27 章/125 项有效，30 个生成视图一致，4 份 Markdown 的 50 个本地链接目标存在，12 个尚未实现能力组和 5 个范围排除组均已记录。独立代理对照 SCOPE 未发现实质问题。测试执行 0 项，未运行回归、构建、CI、网络或音频实验；未提交工作区的检查不替代第 4 步固定提交复验，详情见任务卡。

第一项登记（历史）的 render、validate --git、render --check、差异和范围检查均退出 0：27 章/125 项有效，30 个生成视图一致；仅本项进度改变，模板、ADR 与其他目标未变。测试执行 0 项，未运行回归、构建、CI 或协议实验；不是固定提交上的正式验收，详见任务卡。

Git 准备阶段（历史）的 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项与已有提交引用有效，30 个生成视图一致。测试执行 0 项；未运行 Python/Rust/CLI 回归、构建、远端 CI 或协议实验。检查与一次只读路径错误的说明见任务卡；这些结果不构成本项正式验收。

上一任务：[AC-00-04](tasks/AC-00-04.md) passed，[PR #20](https://github.com/Ray-56/yakyak/pull/20) 已合入 main，主线双平台 CI 成功。固定设计 C `aab3180fb645ed46972d683e7c1654d46e5b7e47`、初次报告 D `00a829f34202cb6b8f6d2870fb07d8b2437c20c3` 和人工登记 `41728e2ea1798a61c47739c0753586e03b3cebd8` 保留；完整结果见[验收报告](../acceptance/ch00/AC-00-04.md)。上一轮按用户“帮我操作一下”的授权完成 Git 与工作区收尾，当时未开始 AC-00-05。

上一轮收尾分支：`docs/implementation-authorization`，工作目录为用户原项目目录；三份既有授权约定改动已原样保存为 `eb567abd2b56cff41980840dce50c917d888e9f1`，与 AC-00-04 的 PR #20 分开交付。旧 AC-00-04 工作树已完整归档后移除；主仓库 Git 历史和初始化工作树保留。该独立文档 PR 在双平台 CI 通过后普通 merge，最终同步回 main；实际最终 SHA、PR、CI 与 clean 核对见下文收尾回执目录。

AC-00-03 的 passed、配置 C `0d972857cc02e51d21d9c02c48dfd6dc40d4d535` 及原始证据不变，完整交付历史见下文与[报告](../acceptance/ch00/AC-00-03.md)。下文各轮“本轮未开始下一 AC”等描述属于对应日期和阶段的历史边界。

2026-10-18（Asia/Shanghai），用户在本会话明确确认 SCOPE v1.0 的功能范围、排除项及自研/复用边界。两个历史 tag 已分别在独立本地 clone 中检出并验证干净，参考 commit 沿用初始化时的真实对象与文件证据。用户授权的 Wireshark 4.0.7 → 4.6.9 更新已完成，旧版保留在仓库外私有备份中。

用户安装软件后，已实测 SIPp 3.8.0-TLS-PCAP、Linphone 6.2.3、Wireshark/TShark 4.6.9 的版本命令；两个 app 的签名和 macOS 信任检查均通过。SIPp `-v` 返回 99，为官方定义的正常无呼叫退出，已保留原始退出码。实际测试端版本缺项已补齐，见 [test-tools-lock.json](../references/test-tools-lock.json)及 [AC-00-01 记录](../acceptance/ch00/AC-00-01.md)。图形界面、协议场景、抓包与音频仍未验证；没有修改权限或系统 PATH。

2026-10-19，按用户授权创建证据提交 A：`41857037d1fda8b7b406534f84aca843db8f68af`，固定本项完整材料；后续提交 B 登记 A 的真实 SHA 与待审阅状态。提交 A 的干净工作区文档检查通过，后续登记的检查结果见验收记录；当时人工审阅字段为空。

随后按用户要求完成技术审阅：AC-00-01 材料齐全，两个旧版本重新独立检出，参考 commit 的四个文件重新取得且哈希一致。但在干净 B `2cea4ed0e4a65ca85953e13d5367ee0c06897e7a` 上运行 `bash scripts/check.sh` 退出 1：21 个 Python 测试中 20 过、1 失败，Rust 检查未执行。已定位为测试只在内存归一化进度、CLI 重读临时磁盘时缺报告的问题。状态回到进行中，详见 [技术审阅记录](../acceptance/ch00/AC-00-01-REVIEW.md)。

用户随后询问修复办法，已用 3 行代码将归一化进度写回测试临时文件，单独保存于 C `b24092bea144d04a42307a76a9b016a3da5c9447`。原失败项红转绿；C 的独立干净副本上完整检查通过：21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建全部成功。DOC-002 已解决，状态恢复待审阅；生产校验与原断言未改变，原始失败证据保留。

2026-10-19，用户在步骤三人工验收说明后明确回复“通过”。已将 AC-00-01 记为 passed，审阅者为“用户（本会话明确确认）”，日期为 2026-10-19；复验提交保持 C，原始证据与历史失败记录保留。

初始化任务 [INIT-001](tasks/INIT-001.md) 已登记 passed。用户先要求补齐记录后自行审阅，随后在收到文档提交 `5cacd60f0c94292dcb4532528cf1f3441cccc805` 及 14 项对照表后回复“没问题”；审阅者为“用户（本会话明确确认）”，日期为 2026-10-19。INIT-11 后续环境证据仍来自主线 `63e8420b37b2cc30348d33d5244a805de60ae3af` 的[双平台 CI](https://github.com/Ray-56/yakyak/actions/runs/39194390065)，各有 21 个 Python、6 个 CLI 测试及格式/Clippy/release 通过。原始 `25157c6…` 在初始化交付时的 Linux/远端 CI not_run 保留，不冒称旧提交已验证。当时 125 项路线图中通过 1 项，其余 124 项为 not_started；INIT-001 不计入该 AC 总数。

## Git 收尾记录（2026-10-19）

初次核对时，本地 `docs/ch00-ac00-01-scope` 的验收登记提交为 `7d366eed0999c850ec2b864f853b2f2a7b19865b`；当时实际远端 `main` 和同名分支都仍为 `cbe33dece07b21b259e393061c4c6d6e19d0e165`，没有对应 PR，工作区干净、无 stash。上述本地验收登记提交已运行 `bash scripts/check.sh` 并退出 0：21 个 Python、6 个 CLI 测试及格式/Clippy/release 构建通过。

用户随后回复“ok”，明确授权推送、创建 PR、等待 Ubuntu/macOS CI 后普通 merge 合入并同步本地 main。已推送 `a5d5179e80a3148a4d549e27699ddf8df699b8c3` 并创建 [PR #14](https://github.com/Ray-56/yakyak/pull/14)；该提交的 [push CI](https://github.com/Ray-56/yakyak/actions/runs/39194109377) 与 [PR CI](https://github.com/Ray-56/yakyak/actions/runs/39194121363) 均在 ubuntu-24.04、macos-15 成功。该 SHA 的本地完整检查也已退出 0（21 个 Python、6 个 CLI 及格式/Clippy/release）。

最终分支提交 `0491442a50ed414cce42f928f027c0ed64074747` 的 push/PR 四项检查全部成功后，PR #14 已用普通 merge 合入，合并提交为 `63e8420b37b2cc30348d33d5244a805de60ae3af`；[主线 CI](https://github.com/Ray-56/yakyak/actions/runs/39194390065) 两平台也成功。本地 main 已 fast-forward 同步，交付时与实际远端 main 一致、工作区干净，验收引用的 C `b24092bea144d04a42307a76a9b016a3da5c9447` 保持祖先可达；历史分支、tag 和 worktree 均保留。最终执行结果保存在忽略的 `artifacts/ac00-01/20261019-git-closeout-vzi79j6w/final-closeout.json`。

## INIT-001 记录补齐（2026-10-19）

基于上述主线建立 `docs/init-001-review-evidence`，按用户授权补齐初始化记录与审阅材料。历史日志位于保留的 `../yakyak-bootstrap-20261016-160959/artifacts/bootstrap/`，已核对 44/44 文件哈希及 28 条原始检查结果；新复核记录位于忽略的 `artifacts/init-001/20261019-review-zdaregbc/`。审阅材料保存于 `5cacd60…`，用户确认后在 `b817778…` 单独登记人工验收结果；报告将历史执行与后续复核分开，三个 ADR 仍 Proposed。

用户随后明确授权 Git 交付，已推送本分支并创建 [PR #15](https://github.com/Ray-56/yakyak/pull/15)。本次补录后的最终 head 须等待 Ubuntu/macOS 全部 CI 成功，再普通 merge 并核对 main CI、本地与远端 main 一致、上述原始提交可达；保留历史分支、tag 和 worktree。实际 head、检查与合并状态以 PR 页面为准，最终执行记录归档至忽略的 `artifacts/init-001/20261019-git-closeout-wwflioo8/`。

2026-10-19 后续核对：PR #15 已于当日 14:41:10（Asia/Shanghai）普通 merge 合入，合并提交为 `bb036b758ee06febbe76266ff73d10a4d9ff43c5`；[main CI 39211342299](https://github.com/Ray-56/yakyak/actions/runs/39211342299) 的 Ubuntu/macOS 均成功。任务准备开始前，本地 main 与实际远端 main 一致、工作区干净；当时已在上述提交上的 `test/ch00-ac00-02` 分支。此前等待合入的安排已完成，原验收提交和历史记录保留。

## AC-00-02 任务准备（2026-10-19）

任务准备阶段，用户明确要求执行“创建任务卡、更新 CURRENT”这一步，并要求不提交代码。当时仅准备 [AC-00-02 任务卡](tasks/AC-00-02.md)，列出原验收、授权边界、已有 CI 与后续测试计划；只修改两份工作文档，`progress.json` 保持 not_started，没有产生实现提交或验收报告。用户随后自行以 `8e11a87` 提交任务文档。

当时的 main CI 各通过 21 个 Python、6 个 CLI 集成测试与格式/Clippy/release 构建；Rust 单元测试目标为 0 项。任务卡将补充有意义的 Rust 单元测试列为后续缺口，旧 CI 仅作为基线。三份 ADR 状态保持 Proposed。

任务准备阶段的文档检查：`python3 scripts/roadmap.py validate --git` 退出 0（27 章、125 项目标及已有提交引用结构有效）；`python3 scripts/roadmap.py render --check` 退出 0（30 份生成文件一致）；`git diff --check` 退出 0。当时没有执行测试用例。

## AC-00-02 实现与待审阅登记（2026-10-19）

用户亲自提取 CLI 参数判定并增加两项 Rust 单元测试，修复开发中的输出、退出码、换行及尾表达式问题后，本地完整检查通过；用户自行以 `9ff5c3fb2dbefa076061cba0a93748f495e66792` 提交并推送。本报告使用固定提交的最终证据，不将未提交草稿的失败或较早工程基线的成功混记为该提交结果。

固定提交的本地独立全新 clone 检查前后均 clean；doctor、完整检查与测试清单均退出 0。随后核对 [push CI 39232969429](https://github.com/Ray-56/yakyak/actions/runs/39232969429)：Ubuntu 24.04.5 x86_64 与 macOS 15.7.9 arm64 均全新检出同一完整 SHA，各通过 21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试，以及格式、Clippy、release 构建。当时实际远端分支与本地被测 SHA 一致，没有 PR、未合入 main。

已整理[验收报告](../acceptance/ch00/AC-00-02.md)，记录原文、环境、命令、实际数量、限制和原始日志 SHA-256；本地复验与 CI 日志保存在忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/`。当时仅将 AC-00-02 登记 ready_for_review，人工审阅字段留空；用户此前表示运行成功、自行实现或推送均未被视为整项人工验收。当时 125 项中 1 项 passed、1 项 ready_for_review、123 项 not_started，其他 AC 与三份 ADR 不变。

登记后的检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章、125 项目标及提交/报告引用有效，30 份生成文件一致，21 个 Python 回归全部通过。原实现固定提交的 Rust 与 release 结果沿用上述完整复验与新 CI，该轮未重复运行；当时新增验收文档尚未提交。

## AC-00-02 人工验收（2026-10-19）

用户在收到验收报告、技术复验结果与明确人工确认说明后回复“通过”。据此将 AC-00-02 登记 passed，人工审阅者为“用户（本会话明确确认）”，确认日期为 2026-10-19（Asia/Shanghai）；被测提交保持 `9ff5c3fb2dbefa076061cba0a93748f495e66792`，原始证据、哈希和 CI 链接保留。

人工验收登记时，125 项中 2 项 passed、123 项 not_started，其他 AC 与三份 Proposed ADR 未改变。当时验收文档尚未提交、没有 PR，未合入 main；该次人工确认本身不包含 Git 交付或开始下一 AC 的授权。

人工确认登记后的检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章、125 项目标及提交/报告引用有效，30 份生成文件一致，21 个 Python 回归全部通过。本轮未重跑 Rust 测试、release 构建或远端 CI，被测提交和既有完整证据保持不变。

## AC-00-02 Git 交付（2026-10-19）

在收到“提交这五份验收记录，再创建 PR”的下一步说明后，用户回复“这些你可以帮我处理”，另行授权 Codex 暂存、提交验收文档，推送当前分支并创建 PR。本轮保留 `9ff5c3fb2dbefa076061cba0a93748f495e66792` 与原始证据，核对交付提交的 Ubuntu/macOS CI；合并不在本次范围内。Git 操作及最终核对记录保留在忽略的 artifacts 目录。

已将五份验收材料保存为 `323fd88d907536df71e42f66fc06606a56cef739` 并推送，创建 [PR #16](https://github.com/Ray-56/yakyak/pull/16)。该材料提交前 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试及格式/Clippy/release 构建通过；`validate --git` 与 diff 空白检查也通过。初次日志重定向使用了不存在的目录，命令退出 1 且检查未启动；随后在正确目录重新运行并成功，没有将路径错误计为测试失败或通过。

补录后的最终 PR head 为 `03600b3e94770a29a57ef8a90443ebc28209aa5c`，push/PR 双平台四项检查全部成功，材料提交和被测实现提交保留。实际 SHA、状态与原始日志归档至忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/git-delivery-58ed1pbl/`。该轮交付结束时 PR 保持打开，Codex 未执行合并、分支删除或下一 AC。

## AC-00-02 合并收尾（2026-10-19）

用户随后自行合并 PR #16，实际合并时间为 2026-10-19 18:18:09（Asia/Shanghai），合并提交为 `78686a8ae1520e61f5ff16241409823495bbd066`。[主线 CI 39236480386](https://github.com/Ray-56/yakyak/actions/runs/39236480386) 的 Ubuntu/macOS 均成功：各 21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试，以及格式、Clippy、release 构建通过。

收尾开始时工作区干净。已 fetch main 并使用 `git merge --ff-only origin/main` 将本地 main 同步到上述实际远端 SHA；被测实现 `9ff5c3f…`、验收材料 `323fd88…` 和 PR head `03600b3…` 均保持祖先可达，未删除任何分支、tag 或 worktree。Git 同步与主线 CI 日志保存在忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/merge-closeout-j7b36ffj/`。补录时四份收尾文档暂留本地，随后用户授权将其在独立 AC-00-02 分支提交并创建 PR。

合并核对阶段的文档检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0；27 章、125 项目标与提交/报告引用有效，30 份生成文件一致，无空白错误。该阶段未重跑本地测试，主线测试数量与结果依据上述 CI 原始日志。

用户指出这四份文档全部属于 AC-00-02，随后回复“好的，帮我处理一下”，授权单独建立 `docs/ch00-ac00-02-closeout`、提交、推送和创建 PR。只处理验收报告、progress.json 的 note、CURRENT 与 AC-00-02 任务卡；不混入 AC-00-03。收尾分支的 PR 与 CI 结果以 GitHub 对应分支记录为准，原始交付记录保存在忽略的 artifacts 中。

独立收尾分支的提交前完整检查 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 集成测试及格式/Clippy/release 构建通过；`validate --git` 与 `git diff --check` 也退出 0。提交前仅上述四份文档存在改动，所有 AC 状态、被测实现与原始证据不变；新提交的 CI 单独核对。原始输出与交付记录保存在忽略的 `artifacts/ch00/9ff5c3fb2dbefa076061cba0a93748f495e66792/docs-closeout-__kh_hjv/`。

## AC-00-03 任务准备（2026-10-20）

开始时本地 main 为 `0bb5d017a791e2834a12efe85b7ed3d33890799f`，与本轮 `git ls-remote` 查询的实际远端 main 一致，工作区干净。此前只读核查已确认 AC-00-02 独立收尾 PR #17 合入，以及 [main CI 39239117830](https://github.com/Ray-56/yakyak/actions/runs/39239117830) 双平台成功；该历史 CI 不能证明 AC-00-03 实验通过，本轮未重查 CI。

按用户请求完成第 1 项：建立 `test/ch00-ac00-03` 分支，创建 [AC-00-03 任务卡](tasks/AC-00-03.md)，记录原验收、已有工具材料、待定拓扑、操作顺序、可观察结果及证据计划，并将 CURRENT 指向该卡。只修改两份工作文档；progress.json 保持 not_started，未创建验收报告或实验产物。

工具版本沿用已有记录，GUI、真实 SIP 场景、抓包权限与音频仍未验证。用户下一步先检查工具可用性；受控端点、端口、抓包接口及场景授权/配额/保留期须在实际实验前具体确定，本轮没有执行网络或音频操作。首次按文件名查找新任务卡时无匹配、退出 1；随后通过任务目录清单确认卡尚未存在，不属于测试失败或通过。

本轮文档检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check` 与 `git diff --check` 均退出 0，27 章、125 项目标结构与提交引用有效，30 份生成文件一致；另用一次性 Python 断言检查两份工作文档的 20 个本地文件链接、行尾空白和文件末尾换行，退出 0。这些是文档检查，未执行 Python/Rust 测试用例、完整基础检查或新 CI，不代表 AC-00-03 实验通过。

## AC-00-03 工具启动反馈（2026-10-20）

用户在收到 SIPp 版本与帮助命令、Linphone 启动、Wireshark 窗口与接口列表的三项手动检查说明后回复“都没有问题”。仅据此补录用户确认工具启动检查无问题；用户未附原始输出，本轮未新增实际版本、退出码或代理独立复验结果，历史工具锁记录保持原样。该反馈不包含实际抓包权限、SIP 收发、音频验证或整个 AC 的人工验收。

任务卡补入本机候选方案：SIPp 发送进程与独立 SIPp 接收进程经 loopback 通信，端口占用、具体地址/端口/协议、抓包接口与范围、场景授权及证据配额/保留期仍待确定。只补录两份工作文档，progress.json 保持 not_started；Codex 未运行 GUI、发包、抓包或音频操作，未提交或推送。

补录后 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0（27 章/125 项结构有效、30 份生成文件一致）；两份文档的 20 个本地文件链接与空白检查也通过。没有运行 Rust/Python 测试用例或实验。

## AC-00-03 第 3 步方案说明（2026-10-20）

用户询问第 3 步如何操作，任务卡已加入本机方案草案：两个独立 SIPp 进程经 UDP `127.0.0.1:15060` → `127.0.0.1:15061` 通信，在 `lo0` 仅按这两个端口捕获；一次正常建立与结束。附加控制口拟用 `127.0.0.1:18888/18889`，后续执行须显式限制 SIP 与控制 socket 的本机绑定，已对照 SIPp v3.8.0 官方源码核对参数。本次不启用媒体播放或 RTP echo。抓包建议上限为 60 秒/10 MiB，原始产物建议保留 7 天，仍待场景授权与具体确认，不自动删除产物。

本轮仅只读核对：`ifconfig lo0` 退出 0，确认 UP/LOOPBACK 与 `127.0.0.1`；`lsof -nP -iUDP:15060 -iUDP:15061` 退出 1 且无输出/错误，对 UDP 18888/18889 的查询也相同，表示当前用户可见范围内没有匹配 socket，启动前须复查。`command -v dumpcap` 退出 1（不在 PATH），随后确认 app 内 `tshark`、`dumpcap` 文件存在，未改环境或权限。未运行发包、抓包、音频或协议测试，progress.json 保持 not_started。

方案补录后，`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0；27 章/125 项结构与引用有效，30 份生成文件一致，两份工作文档的 20 个本地文件链接及空白检查通过。未执行 Rust/Python 测试用例或完整基础检查。

## AC-00-03 回环接口显示核对（2026-10-20）

用户报告 Wireshark 中未看到 `lo0`。只读执行 app 内 `dumpcap -D` 退出 0，明确列出第 17 项 `lo0 (Loopback)`；`ifconfig lo0` 退出 0。由此仅确认系统接口与 helper 枚举正常，尚未确认 GUI 显示原因或实际抓包权限。只读读取 Wireshark 窗口的工具报 ScreenCaptureKit `-3811`，未建立窗口绑定或取得当前界面；没有调整权限、安装组件或开始抓包。下一步核对接口名称、类型筛选与隐藏项，progress.json 保持 not_started。

已按 Wireshark v4.6.9 官方文档核对 Manage Interfaces → Local Interfaces 的 Show 控制接口显隐，具体用户操作路径见任务卡；没有远程修改配置。补录检查 `validate --git`、`render --check`、`git diff --check` 均退出 0（27 章/125 项、30 份生成文件），两份工作文档的 20 个本地链接与空白检查通过；未执行测试用例或实际捕获。

## AC-00-03 BPF 权限错误（2026-10-20）

用户确认 `lo0` 已能找到，随后反馈填写 filter 后报错；询问原文后取得的是设备 `ap1` 的 `/dev/bpf0: Permission denied`。该原文指向打开抓包设备时的权限拒绝，没有过滤器语法错误的证据；报错接口也不是拟用的 `lo0`。本轮只查看 dumpcap 帮助与本机权限，未执行过滤器编译或捕获。

只读核对显示 `/dev/bpf0` 为 root:wheel、`crw-------`，当前用户无读写访问且不在 `access_bpf` 组；标准 ChmodBPF LaunchDaemon 和支持目录未发现。已定位 app 自带的官方 `Install ChmodBPF.pkg`，`pkgutil --check-signature` 退出 0，签名主体 Wireshark Foundation、Apple 公证可信。官方 macOS 安装说明要求实际抓包使用 ChmodBPF；安装会改变机器 BPF 权限，需用户决定后手动操作。本轮没有打开安装器、修改权限、发包或抓包。

后续用户截图显示已选中 `Loopback: lo0`，欢迎页明确提示缺少抓包权限，底部为 `Unable to check capture filter`；当前不应归因为过滤器语法错误。已告知安装包在本机，可由用户点击官方 `installing ChmodBPF` 链接打开并手动确认安装，Codex 未执行安装。该轮文档检查 `validate --git`、`render --check`、`git diff --check` 均退出 0，27 章/125 项、30 份生成文件一致；两份文档的 20 个本地链接与空白检查通过，未执行测试用例或捕获。

## AC-00-03 ChmodBPF 只读复核（2026-10-20）

用户处理后反馈“现在看起来可以了”。只读复核确认标准 ChmodBPF LaunchDaemon 与支持目录已存在；`/dev/bpf0` 为 root:access_bpf、`crw-rw----`，当前工具会话读写访问检查为 True。`launchctl print system/org.wireshark.ChmodBPF` 退出 0、last exit code 为 0，权限脚本执行完成后 exited/not running 属正常结束。权限准备已有实证，实际 `lo0` 捕获尚未运行；Codex 未安装、修改权限、发包或抓包，未将本次反馈登记为场景授权或 AC 人工验收。

补录后 `validate --git`、`render --check`、`git diff --check` 均退出 0（27 章/125 项结构与引用有效、30 份生成文件一致），两份工作文档的 20 个本地链接与空白检查通过；没有运行 Rust/Python 测试用例或实际捕获。

## AC-00-03 第 4 步手动教学（2026-10-20）

用户表示未用过 SIPp，请求讲解如何完成第 4 步。任务卡已加入用户手动指南：新建独立原始记录目录，终端 A 启动内置 uas，Wireshark 限定 lo0/端口与自动停止，终端 B 启动一次内置 uac，两端自动退出后核对实际统计并保存 pcapng 与消息日志。SIP/控制 socket 都显式绑定回环，单次/单并发，主叫 20 秒超时、被叫等待上限 300 秒；图形抓包按 60 seconds/10 megabytes 设置（约 9.54 MiB，低于计划 10 MiB，停止阈值可能多最后一包）。仅准备讲解，不代用户执行场景或捕获，不将教学请求或期望结果记为通过。

本轮仅执行 `sipp -h` 选项核对（退出 99，为帮助命令正常无呼叫退出）及只读端口查询：四个候选 UDP 端口无匹配 socket，lsof 退出 1 且无输出/错误。命令、内置场景与 GUI 参数已对照官方版本资料。首次手动试跑记录不等于固定提交的正式验收；当前指导文档未提交，正式验收仍需固定场景/命令/版本并干净复现。progress.json 保持 not_started，未发包、抓包、录音、提交或推送。

教学补录检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项结构与引用有效、30 份生成文件一致。两份工作文档的 20 个本地链接与空白检查通过；指南的 3 个 bash 代码块均用 `bash -n` 静态解析成功，没有执行块内命令。这些不是 SIPp 场景或实际捕获结果；Rust/Python 测试用例未运行。

## AC-00-03 第 4 步实际结果核对（2026-10-20）

用户完成手动操作并提供 `/Users/ray/Downloads/sip-loopback.pcapng`，反馈“都成功了”。只读 capinfos/TShark 核对确认文件 3140 字节、唯一接口 lo0、原计划过滤器与两个本机端口，6 个 SIP 包属于同一 Call-ID，顺序 INVITE/180/200/ACK/BYE/200。两端退出码文件均为 0，6 条实际 UDP 载荷均与两端消息日志逐字节对应，必要头及 Content-Length 检查通过。本次独立手动信令与抓包结果已确认，不能替代后续音频检查或整个 AC 的人工验收。

保留 Downloads 原文件，在忽略的 `artifacts/ch00/ac00-03/manual-ZTgdQjAj/` 归档同字节副本、真实 INVITE 样例、capinfos 输出与 `verification.json`（14/14 离线核对通过）。原始抓包 SHA-256 为 `9d9473eb780f74b0d7ae57af11cd6632c4c08249bc496de2c3d3a52459115ec2`。未提供最终 Statistics 屏幕；首末包跨度约 1.002462 秒不证明整个捕获窗口或自动停止设置，限制已在摘要与任务卡中说明。

首次临时摘要脚本因时间字段为 ISO 字符串而退出 1，修正离线解析后退出 0，原始抓包未改。只将 AC-00-03 状态从 not_started 更新为 in_progress，待审阅提交、报告与人工审阅字段保持空值，随后用 render 更新生成视图。当前为用户手动试跑证据，尚未固定配置提交或干净复现；未修改核心代码、重跑网络、提交或推送。

登记后 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项结构与引用有效，30 份生成文件一致；实际仅 PROGRESS 生成视图变化。另核对只有 AC-00-03 的源状态与 note 改变，当前 2 项 passed、1 项 in_progress、122 项 not_started；两份工作文档 20 个本地链接和空白检查通过，原始抓包/样例/摘要均被 Git 忽略。本轮没有运行 Rust/Python 回归、release 构建、新 CI 或重跑网络场景。

## AC-00-03 第 5 步音频检查说明（2026-10-20）

用户请求第 5 步操作，并确认 Linphone 已进入联系人/通话主界面。只读 Info.plist 核实版本为 6.2.3，系统音频清单识别默认输入 MacBook Pro Microphone、默认输出 MacBook Pro Speakers；只证明设备被系统识别，未采集或试听。安装包 qml 目录两次文本查找未匹配、退出 1，后改读官方 6.2.3 源码，未读取用户账号配置。

任务卡已加入准确界面路径及手动检查：Application options → Settings → Calls → Devices，选择麦克风/输出设备，用户手动授权麦克风并说话观察 Linphone 实时电平；该版无独立测试音/录制回放按钮，输出另用 macOS 提醒音确认。离开设备页停止本地采集。该操作不需公网号码或保存录音；系统提示音不能证明 Linphone 通话内播放或端到端互通。当前尚未收到实际输入/听音结果，progress.json 保持 in_progress，Codex 未操作 GUI、启用麦克风、录音或呼叫。

首次文档补丁因上下文不匹配被拒绝且未写入，纠正后成功。补录后的 `validate --git`、`render --check`、`git diff --check` 均退出 0（27 章/125 项、30 份生成文件），两份文档 20 个本地链接及空白检查通过。本轮未运行 Rust/Python 测试用例、网络场景或音频实验。

## AC-00-03 第 5 步用户确认（2026-10-20）

用户在收到手动设备检查步骤后回复“都没有问题正常工作”并提供截图。截图直接确认 Linphone Devices 页选择 Default Playback/Default Capture，并有绿色麦克风电平；动态电平随说话变化及系统提醒音听见依据用户手动确认，不冒称静态截图或 Codex 独立听音已证明。只读系统音频清单再次确认复核时默认输入 MacBook Pro Microphone、默认输出 MacBook Pro Speakers，命令退出 0；截图本身只显示默认设备别名。

用户确认与截图文字观察保存在忽略的 `artifacts/ch00/ac00-03/manual-ZTgdQjAj/audio-device-check.json`，SHA-256 为 `8214b01767b046f82f57d937ffd8e35e1c7d57b13dfb3eb2b2ebb28139d160f9`；未保存原始截图、摄像头硬件 ID 或录音，未操作麦克风/摄像头或通话。更新任务卡、progress note 与 LAB-001 的过时描述，in_progress 和空的人工审阅/提交/报告字段保持不变，未将本地设备正常视为通话音频互通或整体 AC 通过。

补录后 `validate --git`、`render --check`、`git diff --check` 均退出 0（27 章/125 项结构与引用有效，30 份生成文件一致），三份工作文档 22 个本地链接与空白检查通过，音频检查文字记录被 Git 忽略。本轮未运行 Rust/Python 回归、网络场景或实际音频采集/播放。

## AC-00-03 第 6 步实际完成（2026-10-20）

固定配置本地提交 C 为 `0d972857cc02e51d21d9c02c48dfd6dc40d4d535`，仅包含本 AC 文档/进度，原始产物未入 Git。以 `git clone --no-local --no-checkout` 创建独立 clone 并 detach 检出 C，前后均 clean。doctor 与完整 check 均退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 测试全部通过，格式/Clippy/release 构建通过。原始日志位于忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/fresh-checkout-bwna4wgq/`。

从 C 任务卡提取既有参数，执行一次同端口的本机 SIPp/lo0 复验：15:33:39.572569–15:33:45.867780 +08:00，编排单调耗时 6.295164 秒。UAC/UAS 各累计成功 1、失败 0，dumpcap 及两端均退出 0，所有子进程结束，四个 UDP 端口可同时重新绑定。抓包 3140 字节、6 个 SIP 包，INVITE/180/200/ACK/BYE/200；双端 12 条发送/接收记录与六个载荷逐字节对应，真实 INVITE 已提取。图形界面、设备反馈沿用用户操作，不冒称本次重测音频。

独立离线核对最终退出 0、19/19 项本样例断言通过；首次离线脚本曾因错误要求 SIPp stderr 为空退出 1，实际为正常 localhost 解析提示，保留失败脚本/摘要后修正，只重跑离线核对，未重跑实验或改动原始证据。完整报告区分用户首次实验、本次干净复验及用户设备确认，并保留历史失败与未运行项。

本次抓包 SHA-256：`355bd9d8b080e437d4e5a97a833c36133b17c03851a8d99be4f9176e43115e74`；真实 INVITE 为 `cb84303c03cb47b86ca69b3139dd77b26107701cd9b96cf27020c3eb6f94d05f`。人工审阅字段保持空，仅登记 ready_for_review；当前 125 项中 2 passed、1 ready_for_review、122 not_started。Rust/测试/工程脚本/依赖与目标要求未改，其他 AC 与 ADR 不变；未推送、创建 PR、合并或开始后续核心任务。

登记材料检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章/125 项与真实提交/报告引用有效，30 个生成视图一致，21 个 Python 回归通过。本轮仅 PROGRESS 生成视图改变。报告关键原始哈希逐一匹配，73 个原始证据文件均被 Git 忽略，新 clone 仍 clean；完整清单在复验目录 manifest.json。两位代理分别只读审查验收覆盖与离线样例，未发现待修正的协议实验问题。登记文档阶段未重复 Rust/release、信令、音频或新 CI。

最终一次性文档核对曾退出 1：报告的命令路径占位符未加引号，无法通过 bash 语法检查；已补正引用方式，再核对语法、文件链接与原始哈希。该检查只解析文档，不执行占位命令或重跑实验。

## AC-00-03 人工验收（2026-10-20）

用户在收到本地材料提交 `0535b7aaa36e7c25237258f1a7fe5ed68e72467d`、完整报告、待人工验收结果及明确整项确认说明后回复“没问题”。据此将 AC-00-03 登记 passed，审阅者为“用户（本会话明确确认）”，日期为 2026-10-20；被测配置保持 `0d972857cc02e51d21d9c02c48dfd6dc40d4d535`，原始证据、哈希和未测媒体范围保留。LAB-001 的本项实验室准备缺口记为 resolved，其他问题、AC 与 ADR 状态不变。

当前 125 项中 3 passed、122 not_started。此次仅补录人工验收记录，尚未提交或推送，不创建 PR、合并或自动开始下一 AC；未重跑网络/抓包、音频、Rust/release 或新远端 CI。

人工验收登记后的检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check` 均退出 0；27 章/125 项及真实提交/报告引用有效，30 个生成视图一致，21 个 Python 回归全部通过。生成视图仅 PROGRESS 改变；原始配置、证据与其他 AC 未改，不冒称本轮重跑完整工程或协议实验。

## AC-00-03 人工验收登记提交（2026-10-20）

用户回复“没啥问题，就提交吧”，授权本地提交六份人工验收登记。仅更新验收报告、progress.json、生成的 PROGRESS、CURRENT、任务卡及 LAB-001 记录；原始产物不入 Git，被测 C 与全部证据引用不变。本次不推送、创建 PR、合并或开始下一 AC，历史提交与分支保留。

提交前 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项及提交/报告引用有效，30 个生成视图一致，无空白错误。此次只提交文档，未重复 Python/Rust 测试、release 构建、网络/音频实验或远端 CI；沿用上轮 21 个 Python 回归及固定 C 的完整复验结果。

## AC-00-03 推送与创建 PR（2026-10-20）

用户在“推送当前分支并创建 PR”的下一步说明后回复“帮我完成1”，授权推送与创建 PR。开始时工作区干净，实际远端 main 为准备基线 `0bb5d017a791e2834a12efe85b7ed3d33890799f`，远端尚无该分支或对应 PR。已推送人工验收提交 `62be8689926ed2f689d8213d433be51a9916e385` 并创建 [PR #18](https://github.com/Ray-56/yakyak/pull/18)，目标 main；本轮不合并、删除分支或开始下一 AC。

交付前在上述 head 执行 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 测试及格式/Clippy/release 构建全部通过；`python3 scripts/roadmap.py validate --git` 与 `git diff --check main...HEAD` 也退出 0。未重跑信令、音频或变更被测 C。范围仅六份脱敏文档/状态文件，原始产物与 LICENSE 未进入变更。

PR 创建后补录本节、任务卡、报告与 progress note，并单独提交推送；最终 head 的 push/PR 双平台 CI 由 GitHub 触发，实际状态以 PR Checks 为准。本轮不将历史 C 的检查外推为新 head 的远端结果。首次 `gh pr view --repo ...` 遗漏显式 PR 编号，查询退出 1；加上 18 后查询成功，没有影响已创建 PR。

原始本地检查、PR 描述及交付核对记录在忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/pr-delivery-t4tt2qih/`。

交付补录的 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项与真实引用有效，30 个生成视图一致。该补录仅改四份文档/进度 note，没有改状态或生成视图；未重复本地完整检查或协议实验，最终 head 的远端检查单独核对。

## AC-00-03 用户合并后的核对（2026-10-20）

用户告知“the pr is merged”。GitHub 实际显示 PR #18 于 15:56:16（Asia/Shanghai）合入，PR head 为 `62be8689926ed2f689d8213d433be51a9916e385`，merge commit 为 `b32cad4cb4b6352d8f6b8bf884eb9bb124da6862`。验收材料与人工 passed 登记均已进入 main；PR head 的 push/PR 四项平台检查均成功。

交付记录补录 `f5f17c576a59713d181089b61b7a4ad56040444b` 在用户合并后才推送，因此未包含于该 PR。原分支和补录提交保留，没有强推、重置、删除或冒称其已合入。原交付核对脚本曾断言 PR 仍 OPEN 且 head 等于本地最新 SHA，因并发合并而退出 1；随后按实际 MERGED 状态与真实合并 head 重新核对，不属于协议或 CI 失败。

[main CI 39389863121](https://github.com/Ray-56/yakyak/actions/runs/39389863121) 在上述 merge commit 的 Ubuntu 24.04/macOS 15 两平台均 success：各 21 个 Python、2 个 Rust 单元、6 个 CLI 测试及格式/Clippy/release 构建通过，0 失败/忽略/过滤。原始 job 日志、run/jobs/summary 与完整性清单在忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/merge-closeout-stxqchh7/`。

已 fetch 并以 `git switch main`、`git merge --ff-only origin/main` 同步本地 main，核对前后 clean、本地与实际远端 main 相同，C、材料提交 `0535b7a…` 和人工验收提交 `62be868…` 均保持祖先可达；f5f17c5 不属于 main 祖先。同步原始记录在忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/merge-sync-fjb781gx/`。

随后建立独立本地 `docs/ch00-ac00-03-closeout`，将未进入 main 的四份交付补录与本次合并事实一起整理，保留真实时序。该轮当时尚未提交、推送或新建 PR，也未合并或开始下一 AC；当时未重跑本地完整检查、信令或音频，主线测试结果依据上述新 CI 原始日志。

收尾文档检查：`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项及提交/报告引用有效，30 个生成视图一致，无空白错误。仅四份文档/进度 note 改变；passed、被测 C、原证据与其他 AC/ADR 不变，未重复本地测试或媒体实验。

## AC-00-03 独立收尾交付（2026-10-20）

用户请求将这四份收尾记录“提交 推送 pr”，并询问完成后能否准备 0004，授权当前独立分支的提交/推送/PR 交付。只处理报告、progress note、CURRENT 与 AC-00-03 任务卡，不改变目标、passed、被测 C、人工审阅信息或原始证据，不混入 AC-00-04 任务卡、ADR 或核心实现。

本轮先完成全部文档再一次提交、推送并创建收尾 PR；最终提交、真实 PR 链接和 CI 状态保存在忽略的 `artifacts/ch00/0d972857cc02e51d21d9c02c48dfd6dc40d4d535/closeout-delivery-0d_x7ou0/`，避免为补 PR 编号继续向已可合并的分支追加提交。最终状态也可从 GitHub 的同名分支 PR/Checks 核对；本轮不执行合并。

本轮提交前 `bash scripts/check.sh` 退出 0：21 个 Python、2 个 Rust 单元、6 个 CLI 测试及格式/Clippy/release 构建通过，27 章/125 项结构有效、30 个生成视图一致；本地链接和四文件变更范围已核对。`python3 scripts/roadmap.py validate --git` 与差异空白检查另行通过。原始日志保存于上述交付目录；未重跑协议、抓包或音频，本项状态和 AC-00-04 的 not_started 不变。

## 实现授权约定补充（2026-10-20）

按用户要求在根目录 AGENTS.md 明确：只有用户明确要求开始实现具体任务时才进入实现阶段，讨论、评审或文档准备不构成实现授权。此次仅修改协作约定及工作记录，AC 状态不变；下一步等待用户明确安排。`git diff --check` 退出 0；未运行测试（0 项），未运行构建或协议实验，无新增运行时风险。

## AC-00-04 设计确认与文档落地（2026-10-20）

用户确认的是完整设计草稿：状态机与执行器分工、消息/定时器/结果事件、两种时间与虚拟时钟、旧定时器及迟到结果失效、无流量停止与 join、共享资源与协议保留状态，以及八类后续测试设计。内容保存于 [ADR-0002](../architecture/adr/0002-protocol-runtime-time.md)，用户确认记录随 ADR 保存，架构目录同步标注 Accepted；ADR-0001 和 ADR-0003 不变。

本轮建立单一 [AC-00-04 任务卡](tasks/AC-00-04.md)，只将 progress.json 中本项改为 in_progress，并由 roadmap.py 生成进度视图。implementation_commit、evidence、reviewer、reviewed_at 保持空值，不以准备基线冒充包含本设计的提交，不把聊天确认扩写为整项 AC 已验收。未修改 targets.json、源码、依赖或测试实现。

本轮在未提交文档上执行 `python3 scripts/roadmap.py render`、`validate --git`、`render --check`、`git diff --check`，均退出 0：27 章/125 项结构与现有引用有效，30 个生成视图一致；只有 PROGRESS 的本项状态与汇总发生变化。`python3 -m unittest discover -s scripts -p 'test_*.py' -v` 退出 0，21 项文档工具测试通过、0 失败。收尾补录后复查前三类只读检查，日志加 `final-` 前缀。原始结果位于独立工作树忽略的 `artifacts/ch00/ac00-04/20261020-design-nf_i1ijb/`，任务卡记录完整命令及首轮补丁格式失败的修正。

本轮没有运行 Rust/CLI 测试、格式/Clippy/release 构建、远端 CI 或网络/音频实验；ADR 的八类协议场景仍为测试设计。首次在独立工作树整理时，原工作区三份未提交改动保持原样；本轮将分支移回原目录后，AGENTS.md 与 AC-00-03 任务卡仍逐字节不变，CURRENT 的用户补充已完整合并。其中新的实现授权约定同样遵守。本轮只处理确认后的设计文档，不将其视为核心实现授权。

## AC-00-04 分支回到原项目目录（2026-10-20）

用户要求“分支没有操作，操作一下”，本轮先备份原工作区三份改动、独立工作树六份设计文档及双方差异，再让原独立工作树保持 detached HEAD，释放已有分支，最后在原项目目录执行 `git switch docs/ch00-ac00-04`。两处 HEAD 均保持准备基线，未产生提交，也未删除分支、工作树或历史。

六份设计文档及原始检查产物已转回当前目录；CURRENT 合并用户的“实现授权约定补充”原文，AGENTS.md 与 AC-00-03 任务卡内容未改。原始备份与本轮检查保存到忽略目录 `artifacts/ch00/ac00-04/20261020-branch-transfer-x2klwzwc/`。设计内容、Accepted 与 in_progress 不变，不把分支切换记成验收通过。

本轮 `python3 scripts/roadmap.py render`、`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项结构与现有引用有效，30 个生成视图一致，无空白错误。未重复 Python 回归（沿用前轮 21 项通过的记录），未运行 Rust/CLI 测试、构建或网络/音频实验。本轮检查不构成固定提交上的正式验收。

下一步固定本地设计提交、在干净提交上复核并准备验收报告；用户后续审阅整项报告。此前对设计的确认已经记录，不要求用户重复确认同一设计；本轮仅操作分支和文档，没有自行提交或推进验收状态。

## AC-00-04 本地固定与干净复核授权（2026-10-20）

用户在本地设计提交、干净复核与验收报告的说明后回复“好的”。本轮按任务卡先提交六份设计/工作文档形成 C，再以新 clone 检出完整 C，运行文档校验、生成一致性、21 项文档工具回归及差异检查；实际 SHA 和检查结果在执行后由验收报告及后续提交 D 登记，不预写通过或 self-reference。

只暂存本项改动；用户既有 AGENTS.md、AC-00-03 任务卡与 CURRENT 的“实现授权约定补充”留在原工作区，不纳入本 AC 提交。CURRENT 的任务记录按内容选择暂存，用户补充与备份保留。不实现通信内核，不运行网络、抓包、音频或远端 CI，不推送、创建 PR 或合并。复核材料齐全后最多 ready_for_review，等待用户审阅整项报告。

## AC-00-04 固定提交复核与待审阅登记（2026-10-20）

已创建本地设计提交 C `aab3180fb645ed46972d683e7c1654d46e5b7e47`，只含本项六份文档；用户三份既有改动继续保留，未纳入 C。使用 `git clone --no-local --no-checkout` 新建独立副本，再明确检出完整 C，检查前后 status 均空。

C 上 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`python3 -m unittest discover -s scripts -p 'test_*.py' -v`、`git diff --check`、`git show --format= --check <C>` 均退出 0：27 章/125 项有效、30 个生成视图一致，21 项 Python 测试通过、0 失败。独立代理只读设计评审六项关注点未发现原 AC 的实质遗漏；不是人工整项验收。

首轮证据汇总数量正则过度转义，汇总退出 1；修正后仅重跑汇总退出 0，五项检查和原始输出不变。原始环境、命令、前后状态、技术评审、错误回执和哈希位于忽略的 `artifacts/ch00/aab3180fb645ed46972d683e7c1654d46e5b7e47/design-review-i3dxcz0h/`，详细结果见报告。没有运行 Rust/CLI 测试、构建、远端 CI、网络/抓包/音频或八类协议测试场景。

验收报告、进度、生成视图、任务卡和 CURRENT 在后续本地 D 单独登记，不改变 C 或 ADR 内容。登记检查日志放原始证据目录的 `registration/`；implementation_commit 指向 C、evidence 指向报告、状态为 ready_for_review，reviewer/reviewed_at 保持空值。登记前原工作区的 `validate --git`、`render --check`、21 项 Python 回归和 `git diff --check` 另行全部退出 0；登记这一轮与 C 的干净结果区分，不冒称原工作区无用户改动或 D 已运行完整工程检查。补齐说明后复查文档/差异检查再提交 D。用户原有 AGENTS.md、AC-00-03 任务卡与 CURRENT 授权补充不纳入 D，本轮不 push/PR/merge 或开始 AC-00-05。

## AC-00-04 整项人工验收确认（2026-10-20）

用户收到固定 C、报告提交 D、完整报告链接、复核结果及明确整项确认提示后回复“通过”。本次据此登记 passed，reviewer 为“用户（本会话明确确认）”，reviewed_at 为 2026-10-20。此前只确认 ADR 内容的记录与本次整项确认分开保留。

仅修改报告、progress.json、生成的 PROGRESS、任务卡与 CURRENT，保存本地人工验收登记；C、原始证据、历史失败记录和未测限制不变。用户既有 AGENTS.md、AC-00-03 任务卡与 CURRENT 补充继续保留，不混入登记提交。`python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0，27 章/125 项及引用有效、30 个生成视图一致。本轮测试执行 0 项，没有重复 Python/Rust/CLI、构建、CI 或协议/音频实验；首轮补丁上下文失败与修正见任务卡。登记检查、授权回执与备份位于忽略的 `artifacts/ch00/aab3180fb645ed46972d683e7c1654d46e5b7e47/human-acceptance-gd2xhlyc/`。不 push/PR/merge 或开始 AC-00-05。

## AC-00-04 推送与创建 PR 交付（2026-10-20）

用户在“推送当前分支并创建 PR”的说明后回复“可以”，授权本轮远端交付。开始时实际远端 main 为准备基线 `8861c04fbd6dfd00805feb18f73bf0e1208ed180`，远端尚无该分支或对应 PR；本地已有设计、证据和人工登记三个提交。

交付前新建本地 clone 检出人工登记提交 `41728e2ea1798a61c47739c0753586e03b3cebd8`，前后 clean；`bash scripts/doctor.sh`、`bash scripts/check.sh` 均退出 0：21 项 Python、2 项 Rust 单元、6 项 CLI 及 fmt/Clippy/release 构建通过，27 章/125 项与 30 个生成视图有效。该轮是后续交付预检，不外推为 C 的新增功能结果，未运行协议、网络、抓包或音频场景。

本轮先写完 CURRENT 与任务卡两份交付记录，单独提交后一次推送、创建 PR，再核对最终 head 的 CI。最终 SHA、PR 链接、CI 运行、原始日志与完整性回执在忽略目录 `artifacts/ch00/aab3180fb645ed46972d683e7c1654d46e5b7e47/pr-delivery-yk_oabgr/`；不为回填 PR 编号向已交付分支反复追加提交。CI 实际状态以该 head 的 Checks 和回执为准，不预写成功。

只追加交付记录，不改 passed、C、人工审阅字段、原始证据、其他 AC/ADR 或目标。用户既有 AGENTS.md、AC-00-03 任务卡和 CURRENT 授权补充仍不纳入提交；不推送这些工作区改动，不 merge 或开始 AC-00-05。

## AC-00-04 合并与工作区收尾（2026-10-20）

用户在合并 PR #20、另交付三份授权约定改动、备份并清理旧工作树的说明后回复“帮我操作一下”。PR #20 已普通 merge，合并提交为 `dd72b3fd9bc4160bb347ef88fc6f81dfdcfcc796`（2026-10-20 18:29:00，Asia/Shanghai）；本地 main 已 fast-forward 同步。其 [main CI 39408251000](https://github.com/Ray-56/yakyak/actions/runs/39408251000) 的 Ubuntu 24.04/macOS 15 均成功：各 21 项 Python、2 项 Rust 单元、6 项 CLI 测试，以及格式、Clippy、release 构建通过。这是合并提交的基础工程检查，不改写设计 C 的原始复核范围。

旧 `../yakyak-ac00-04` 是早期设计的 detached 工作树，不是另一套待实现代码。其 123 个文件（633697 字节，含未提交文档、原始产物和缓存）已完整归档并逐文件核对内容、权限；独立代理只读复核确认无遗漏。删除前再次核对一致后，用 `git worktree remove --force` 移除该工作树。没有删除历史分支、tag、主仓库 Git 数据或 `../yakyak-bootstrap-20261016-160959`。

收尾回执与备份目录：`artifacts/ch00/aab3180fb645ed46972d683e7c1654d46e5b7e47/workspace-closeout-2ksiub4p/`。完整旧工作树在其中 `archived-ac00-04-worktree/`；`old-worktree-manifest.json` 记录哈希与权限，`archive-restore.txt` 说明恢复方法。归档中的 `.git` 是旧工作树指针，恢复时应创建新工作树再复制内容，不能直接视为独立仓库。原始证据仍受 Git 忽略，保留且不自动删除。

此前留在工作区的 AGENTS.md、CURRENT 授权补充和 AC-00-03 任务卡原文，已单独固定为上述授权约定提交；本轮只再更新 CURRENT 与 AC-00-04 任务卡。先完成本次记录，再提交、推送并创建独立 PR；其最终 head 的 push/PR 双平台检查、普通合并、主线 CI 与同步结果只登记于上述回执目录，避免为回填编号反复追加提交。AC-00-04 的 passed、C、人工审阅与原始证据，以及其他 AC/ADR/目标均不变。

本轮本地 `python3 scripts/roadmap.py validate --git`、`python3 scripts/roadmap.py render --check`、`git diff --check` 均退出 0：27 章/125 项与提交引用有效，30 个生成视图一致，无空白错误。只核对文档，本地测试执行 0 项，没有重跑 Rust/CLI、构建或协议/音频实验；新 head 和合并后 main 的完整基础检查由实际 CI 单独核对。

## 后续安排

执行前冻结记录（2026-10-20，历史）：用户回复“6 应该你可以帮我处理了”。当时按任务卡冻结现有 UAC/UAS 命令、工具版本与内置 XML 哈希，计划新 clone 运行 doctor/完整基础检查及一次相同端口的有界 lo0 抓包实验，最长 60 秒、低于 10 MiB，原始产物保留 7 天且不自动删除。仅操作本轮子进程，不安装软件或放宽权限，不开麦或录音；此前设备用户确认单独引用。当时尚未复验，状态仍 in_progress，先创建本地固定提交 C 再补录结果；现已完成，实际结果见上节，不 push/PR/merge。

1. 第 6 步已完成本地固定提交、干净复验与完整报告。原始产物保留 7 天，至 2026-10-27 由用户决定，不自动删除。
2. AC-00-04 的固定 C、干净复核、报告、人工验收及 PR #20 合入已完成。本轮独立交付授权约定与收尾记录，核对 CI、合入并同步 main；最终结果见收尾回执。
3. ADR-0002 保持 Accepted；ADR-0001、ADR-0003 仍 Proposed，另行审阅。本次确认只覆盖 AC-00-04，不自动开始 AC-00-05。
4. 后续 AC 需用户明确安排，不因本项 passed 自动开始下一项；逐项完成第 00 章后再安排 AC-01-01。

当前 AC-00-04 为 passed，设计评审与人工验收已完成，通信内核未实现。AC-00-03 保持 passed，原始信令与设备确认沿用原报告，通话内媒体仍未测。问题与解除条件见 [KNOWN_ISSUES.md](KNOWN_ISSUES.md)。

同步说明：远端 53 个旧实现提交通过双亲合并保留，活跃树采用初始化结果。原 README 未提交修改已保存到仓库外私有备份，详情见初始化报告“后续授权：直接同步 main”一节。
