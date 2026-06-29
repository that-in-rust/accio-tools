# Tool Routing Benchmark Deep Dive
## Premise Check
- Existing benchmarks do cover much of what we need: pure tool retrieval, MCP-style multi-tool tasks, API/tool-use labels, function selection, and irrelevance cases.
- They do not give a perfect drop-in for the take-home prompt: arbitrary user-connected enterprise MCP servers with sparse metadata, overlapping product terms, and a tiny runnable harness. The fastest credible move is to use existing benchmark data as the query/gold source, then evaluate our router on a 50-query subset.

## Expert Lenses
- Benchmark specialist: prefer ToolRet for router-only evidence because it directly evaluates tool retrieval.
- MCP implementer: prefer LiveMCPBench and MCP-Bench because they look like real connected MCP tool environments.
- Take-home reviewer: values a defensible, runnable eval more than a massive imported benchmark.
- Skeptical engineer: warns that BFCL/ToolBench/AppWorld/tau2-bench often evaluate tool calling or task success, not the pre-model routing decision itself.

## Candidate Approaches
1. Run ToolRet only: strongest router-only benchmark, but not MCP-shaped enough for the prompt.
2. Run LiveMCPBench/MCP-Bench only: strongest MCP shape, but heavier and more end-to-end than a 5-hour take-home needs.
3. Use a curated 50-query subset from existing benchmarks: fastest path that still covers the important failure modes.

## Chosen Thesis
Use the curated 50-query subset as the main evaluation and cite ToolRet as the closest existing benchmark. This avoids building a benchmark from scratch while keeping the test relevant to arbitrary tool routing.

## Evidence and Verification
- Cloned benchmark/reference repos are recorded below.
  - `benchmarking-tool-retrieval`: `https://github.com/mangopy/benchmarking-tool-retrieval.git` at `c4181d9`, shallow=True path `git-ref-repo/benchmarking-tool-retrieval`
  - `LiveMCPBench`: `https://github.com/icip-cas/LiveMCPBench.git` at `36a51a1`, shallow=True path `git-ref-repo/LiveMCPBench`
  - `mcp-bench`: `https://github.com/Accenture/mcp-bench.git` at `7a8eaea`, shallow=True path `git-ref-repo/mcp-bench`
  - `gorilla`: `https://github.com/ShishirPatil/gorilla.git` at `6ea5797`, shallow=True path `git-ref-repo/gorilla`
  - `StableToolBench`: `https://github.com/THUNLP-MT/StableToolBench.git` at `aa4ed9f`, shallow=True path `git-ref-repo/StableToolBench`
  - `ToolBench`: `https://github.com/OpenBMB/ToolBench.git` at `d56fdd8`, shallow=True path `git-ref-repo/ToolBench`
  - `tau2-bench`: `https://github.com/sierra-research/tau2-bench.git` at `8ebb749`, shallow=True path `git-ref-repo/tau2-bench`
  - `appworld`: `https://github.com/StonyBrookNLP/appworld.git` at `a072b7a`, shallow=True path `git-ref-repo/appworld`
  - `ToolSandbox`: `https://github.com/apple/ToolSandbox.git` at `165848b`, shallow=True path `git-ref-repo/ToolSandbox`
- 50-query subset saved at `A00-raw-research/tool-routing-50-query-candidates.json`.
- Source mix: `{'BFCL': 4, 'LiveMCPBench': 12, 'MCP-Bench': 18, 'StableToolBench': 8, 'ToolBench': 5, 'ToolRet': 3}`.
- ToolRet source: `benchmarking-tool-retrieval` repo plus Hugging Face datasets `mangopy/ToolRet-Queries` and `mangopy/ToolRet-Tools`.

## Recommended 5-Hour Evaluation Plan
1. Load the 50-query JSON.
2. Convert each `gold_tools_or_apis` entry into the router gold set. For MCP-Bench rows, use extracted `Server:tool` mentions; if sparse, evaluate at server-level first.
3. Compare three methods: full tool set, keyword/BM25 top-k, and our router top-k.
4. Report `Recall@K`, `Precision@K`, average selected tool count, and token reduction. Treat recall as primary because missing the required tool is worse than including a few extras.
5. In the design note, say this is a benchmark-derived eval subset, not a newly invented synthetic benchmark.

## 50 Query Index
| id | source | source id | gold / label summary | query |
| --- | --- | --- | --- | --- |
| Q01 | LiveMCPBench | `0e3287cb-c0ff-4d2a-8c3d-d8833014a7b0` | 1. get-weread-rank 2. generate_word_cloud_chart 3. create_document 4. add_paragraph 5. add_picture 6. convert_to_pdf | Generate a well-formatted PDF report titled wechat_reading_report.pdf in /root/pdf, summarizing current WeChat Reading trends and including a word cloud. |
| Q02 | LiveMCPBench | `99d7541a-73c0-4ea1-8002-a70a041f3ffc` | 1. load 2. beat_track 3. mfcc 4. chroma_cqt | Please perform a comprehensive analysis of the audio file located at /root/music/mixkit-retro-game-emergency-alarm-1000.wav. The analysis should include beat tracking,... |
| Q03 | LiveMCPBench | `dda8ea55-629c-4ae8-ab87-0c81477ae615` | 1. geocode_address 2. find_parking_facilities 3. find_nearby_places 4. reverse_geocode | I'm planning a trip to the Forbidden City in Beijing, show me where to park and what's interesting to do within 1km distance? |
| Q04 | LiveMCPBench | `629106a5-9508-4ac9-97c3-f22f6069ee7f` | 1. valorant-characters-statistics | My goal is to reach Radiant in Valorant, what should I focus on to improve and start climbing? |
| Q05 | LiveMCPBench | `5a2720cd-20f1-49e7-a3fc-8e74af2217f6` | 1. geocode_address 2. suggest_meeting_point 3. find_nearby_places 4. reverse_geocode | There are three of us. One person is in Haidian Huangzhuang, another is at Chaoyang Joy City, and the third is at Shijingshan Hospital. Could you please recommend a fa... |
| Q06 | LiveMCPBench | `e9df60b1-37a6-4b10-aff0-d432fa79d400` | 1. search_papers 2. get_paper_data | Please help me find the latest articles about 'MCP' on arXiv. Identify the potentially most influential papers among them and analyze the current trends in this field. |
| Q07 | LiveMCPBench | `2b96c022-a2db-47a5-981a-03cc6ad8f95e` | 1. whois_domain 2. get_sitemap_tree 3. write_file | Run a full reconnaissance on www.baidu.com. Give me a markdown report on /root/markdown/baidu_report.md with whois information and sitemap. |
| Q08 | LiveMCPBench | `88ccea26-01f6-4db6-ba95-6b2a73c0488b` | 1. get_historical_stock_prices 2. get_news 3. get_recommendations 4. create_presentation 5. save_presentation | Analyze Apple's stock for the past year and summarize it in a PPT in /root/ppt/apple.pptx. |
| Q09 | LiveMCPBench | `230fb273-b316-4a8f-9297-89ce5558bc6c` | 1. get-daily-challenge 2. write_file | Find today's LeetCode Daily Challenge and generate a detailed markdown document in /root/markdown/leetcode_daily.md that provides a comprehensive walkthrough of the pr... |
| Q10 | LiveMCPBench | `f2a9eb67-efec-4c86-abad-efa55a72f86d` | 1. research-with-keywords 2. write_file | Generate a deep report of bitcoin and save it in /root/markdown/bitcoin.md |
| Q11 | LiveMCPBench | `08644b1a-ac5e-45a9-8ad1-3a3942e8d366` | 1. get-smzdm-rank 2. excel_write_to_sheet | Help me recommend any cheap goodies worth buying recently? Save this information to /root/excel/goods.xlsx. |
| Q12 | LiveMCPBench | `3c65e618-2ed0-444b-b7c7-6135d3841976` | 1. mcp_howtocook_whatToEat 2. mcp_howtocook_getRecipeById | Recommend me a combination of dishes for a potluck dinner for three people and then tell me what ingredients I need to prepare in total and how each dish should be pre... |
| Q13 | MCP-Bench | `openapi_explorer_000` | OpenAPI Explorer | Hey, I’m working on this new dashboard that pulls search results from three different services—one for AI stuff, one for code hosting, and one for edge networking—and ... |
| Q14 | MCP-Bench | `openapi_explorer_001` | Call OpenAPI Explorer:getApiOverview, OpenAPI Explorer:getApiOperation | Hey, I’m building a little integration for my team and could really use a sanity check on two services we’re about to hook up. One of them is an AI platform where most... |
| Q15 | MCP-Bench | `unit_converter_000` | Unit Converter | Hey, I’m prepping for a Reactor X startup tomorrow and it’s stressing me out a bit. My boss handed me 14 different sensor readings, all in weird units, and I need to k... |
| Q16 | MCP-Bench | `unit_converter_001` | Unit Converter | I’m gearing up for a 2 h 30 min high-altitude drone test flight and my boss wants every single detail in SI. Right now all my numbers are in U.S. customary units: engi... |
| Q17 | MCP-Bench | `wikipedia_000` | Call Wikipedia:extract_key_facts, Call Wikipedia:get_article, Call Wikipedia:get_links, Call Wikipedia:get_related_topics, ... (+6) | I’m prepping for a presentation on the big global climate deals and could really use some solid data. Could you find the main half-dozen—or so—negotiation frameworks t... |
| Q18 | MCP-Bench | `wikipedia_001` | Wikipedia | I’m putting together a sustainability briefing and need to really understand how solar panels stack up against wind turbines when it comes to environmental impacts—thi... |
| Q19 | MCP-Bench | `paper_search_biomcp_000` | Also use Paper Search:search_pubmed, BRAF inhibitors in melanoma with BioMCP:trial_searcher, BRAF with BioMCP:gene_getter, BioMCP:article_getter, ... (+10) | I’m working on a project about why melanoma patients with the BRAF V600E mutation so often become resistant to treatment, and I’m a bit stuck piecing everything togeth... |
| Q20 | MCP-Bench | `paper_search_biomcp_001` | BRAF inhibitors via BioMCP:trial_searcher, BRAF therapy with Paper Search:search_biorxiv, BRAF with BioMCP:gene_getter, BioMCP:article_getter, ... (+10) | Hey, I’m prepping a report for my boss on BRAF V600E in melanoma and I’ve hit a wall. I really need to know how common that V600E swap is and what clinical impact it a... |
| Q21 | MCP-Bench | `wikipedia_nasa_data_000` | Call Wikipedia:extract_key_facts, Call Wikipedia:get_related_topics, Call Wikipedia:summarize_article_for_query, Using NASA Data:get_notifications, ... (+1) | I’m working on a little side project for my boss about last week’s space weather chaos. I know we had a mix of solar flares, CMEs, geomagnetic storms, particle events,... |
| Q22 | MCP-Bench | `wikipedia_nasa_data_001` | Call NASA Data:get_asteroids_feed, Call Wikipedia:extract_key_facts, Call Wikipedia:get_article, Call Wikipedia:search_wikipedia, ... (+3) | I’ve got this astronomy assignment for my research group, and it’s been bugging me: I need to know which near-Earth rocks swung by in the past week, and whether any bi... |
| Q23 | MCP-Bench | `google_maps_national_parks_000` | CO to the campground using Google Maps:maps_directions, Google Maps:get_place_details, Google Maps:maps_distance_matrix, Google Maps:maps_elevation, ... (+8) | Hey, I’m trying to nail down a three-day hiking and waterfall road trip next week, starting from Denver. Ideally I want parks within about a five-hour drive where ther... |
| Q24 | MCP-Bench | `google_maps_national_parks_001` | Call National Parks:getAlerts, Call National Parks:getCampgrounds, Call National Parks:getEvents, Call National Parks:getParkDetails, ... (+10) | I’ve been itching to head out of Denver for a 5-day camping trip sometime in the next week, but I’m kind of torn on which national park makes the most sense. Ideally i... |
| Q25 | MCP-Bench | `google_maps_weather_data_national_parks_000` | Google Maps+Weather Data+National Parks | Hey there—I’m gearing up for a quick three-day camping getaway to Yosemite from San Jose and, to be honest, I’m feeling a bit swamped by all the options and details. I... |
| Q26 | MCP-Bench | `google_maps_weather_data_national_parks_001` | Google Maps+Weather Data+National Parks | I’m trying to plan a week-long hiking and camping loop that starts and ends in Denver, and I’m hoping you can really nerd out with me on the details. I want to hit a f... |
| Q27 | MCP-Bench | `hugging_face_paper_search_wikipedia_001` | Call Hugging Face:get-dataset-info, Call Paper Search:download_arxiv, Hugging Face:get-model-info, Paper Search:search_arxiv, ... (+5) | I’m working on a project where I need to pick the very best news‐article classifier out there right now—specifically the one built for that 4-category news dataset (wo... |
| Q28 | MCP-Bench | `hugging_face_paper_search_wikipedia_003` | Hugging Face+Paper Search+Wikipedia | Hey, I’m working on a little side project where I need a solid German→English translation model, and I’ve heard the Helsinki team has some of the best open-source opti... |
| Q29 | MCP-Bench | `paper_search_call_for_papers_wikipedia_000` | Wikipedia:get_summary | Hey there! I’m working on a project to build a real-time pandemic outbreak detector, and I want to see what cutting-edge machine learning tricks have popped up in the ... |
| Q30 | MCP-Bench | `paper_search_call_for_papers_wikipedia_002` | Paper Search+Call for Papers+Wikipedia | I’m prepping for a journal club talk on CRISPR gene editing and keep hearing about new ways to predict Cas9 off-target effects with machine learning. I’ve been hopping... |
| Q31 | StableToolBench | `588` | ["TheClique", "Transfermarkt search"], ["TheClique", "Transfermarkt details"] | I'm a football enthusiast and I want to know more about Lionel Messi's career. Can you provide me with information about Messi's clubs, managers, teammates, and refere... |
| Q32 | StableToolBench | `608` | ["Kick.com API / Kick API", "Get Channel Clips"], ["Kick.com API / Kick API", "Get Channel Details"] | I'm a fan of the streamer 'gmhikaru' on Kick.com and I want to explore their channel. Can you provide me with the channel clips and the channel details? I'm eager to l... |
| Q33 | StableToolBench | `1073` | ["Keyword Analysis", "PopularSitesForQuery"], ["Keyword Analysis", "QueryKeywords"] | I want to plan a surprise birthday party for my friend. Can you suggest popular sites and main keywords for the search query 'birthday party ideas'? I want to gather i... |
| Q34 | StableToolBench | `1572` | ["public-url-share", "Media sources statistics"], ["public-url-share", "Top 50 startups news from the last month"] | I'm curious about the credibility and popularity of different media sources. Can you fetch me the statistics on media sources? Additionally, I'm interested in reading ... |
| Q35 | StableToolBench | `1856` | ["Morpheus Predictions ", "Best BestGoal"], ["Morpheus Predictions ", "Best Sign 2"], ["Morpheus Predictions ", "Best Sign 1"], ["Morpheus Predictions ", "Best Quote"] | I'm planning to place some bets on today's soccer matches. Can you provide me with the best goal prediction, the best 2 sign prediction, and the best 1 sign prediction... |
| Q36 | StableToolBench | `2121` | ["Live Golf Data", "World Ranking"], ["Live Golf Data", "points"], ["Live Golf Data", "scorecards"] | I'm planning a golf tournament and I need information about the players. Can you provide me with the world rankings of the top players for the year 2022? Additionally,... |
| Q37 | StableToolBench | `2144` | ["F1 drivers quotes", "Driver's quotes by ID"], ["F1 drivers quotes", "Author with his photo"] | I am organizing a Formula 1 themed party and I need some quotes by Daniel Ricciardo to put up as decorations. Can you fetch a specific quote by its ID? The driver ID i... |
| Q38 | StableToolBench | `2213` | ["RugbyAPI2", "PreMatchForm"], ["RugbyAPI2", "MatchIncidents"] | My company is hosting a rugby event and we need to provide pre-match form information to the participants. Can you fetch the pre-match form for a specific rugby match?... |
| Q39 | ToolBench | `1` | ["SQUAKE", "Checkhealth"], ["SQUAKE", "Projects"] | I'm working on a logistics project for my company and need to check the health of the SQUAKE API. Can you verify the API health by calling the 'Checkhealth' API endpoi... |
| Q40 | ToolBench | `2` | ["SQUAKE", "Checkhealth"], ["SQUAKE", "Projects"] | As a sustainability enthusiast, I'm interested in exploring the projects available on the SQUAKE platform. Could you fetch the list of projects using the 'Projects' AP... |
| Q41 | ToolBench | `3` | ["SQUAKE", "Checkhealth"], ["SQUAKE", "Projects"] | I'm organizing a sustainability event for my community and would like to showcase various projects promoting eco-friendly practices. Could you provide me with a list o... |
| Q42 | ToolBench | `4` | ["SQUAKE", "Checkhealth"], ["SQUAKE", "Projects"] | My friend and I are passionate about sustainability and want to contribute to projects that promote eco-friendly practices. Can you fetch the list of projects availabl... |
| Q43 | ToolBench | `5` | ["Transportistas de Argentina", "/tracking/correo_argentino/create_task/:service/:tracking_code"], ["Transportistas de Argentina", "/tracking/correo_argentino/result_task/:task_id"] | I need to track a package sent through Correo Argentino. The tracking code is 'ABC123'. Please create a task to get the history of the package and provide me with the ... |
| Q44 | BFCL | `multiple_0` | triangle_properties.get, circle_properties.get | Can I find the dimensions and properties of a triangle, if I know its three sides are 5 units, 4 units and 3 units long? |
| Q45 | BFCL | `multiple_1` | math.triangle_area_heron, math.circle_area, math.triangle_area_base_height | Calculate the area of a triangle, given the lengths of its three sides: 3, 4, and 5. |
| Q46 | BFCL | `irrelevance_0` | determine_body_mass_index | Calculate the area of a triangle given the base is 10 meters and height is 5 meters. |
| Q47 | BFCL | `irrelevance_1` | math.sum | Solve the quadratic equation with coefficients a = 1, b = 2, and c = 3. |
| Q48 | ToolRet | `toolbench_query_808` | Onboarding_Project_v3_Get_User_Orders, Onboarding_Project_v3_Get_Order | I want to check the status of my recent order. Can you fetch the details of my order, including the order status and delivery date? |
| Q49 | ToolRet | `toolLens_query_0` | /topic/{topic}, Search/Autocomplete API, getQuestionsandAnswers | I'm setting up a learning management system for my online courses and need information based on the coordinates 34.0522,-118.2437. |
| Q50 | ToolRet | `metatool_which_query_0` | airqualityforeast | I'm planning to take my kids to the park tomorrow. Can you provide the air quality forecast for zip code 98101? |

## Open Questions
- Do we want query-level gold to mean exact tool IDs, server IDs, or both? For the take-home, I would report both when possible, but optimize exact-tool recall.
- Do we evaluate final agent success, or router quality only? For a 5-hour build, router quality plus a tiny harness smoke test is enough.
