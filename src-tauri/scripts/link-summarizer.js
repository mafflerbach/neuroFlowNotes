#!/usr/bin/env node

/**
 * Link Summarizer Script
 *
 * Scrapes web pages using Puppeteer and summarizes them using a local LLM.
 * Outputs JSON status messages to stdout for integration with NeuroFlow.
 *
 * Usage:
 *   node link-summarizer.js --output-dir <path> [--endpoint <url>] [--model <name>] <url1> [url2] ...
 *   node link-summarizer.js --output-dir <path> --file <links.txt>
 */

const puppeteer = require('puppeteer');
const fs = require('fs').promises;
const path = require('path');
const axios = require('axios');
const os = require('os');

// JSON output helper for NeuroFlow integration
function emit(type, data) {
    console.log(JSON.stringify({ type, ...data }));
}

class LinkSummarizerAgent {
    constructor(options = {}) {
        this.lmStudioEndpoint = options.lmStudioEndpoint || 'http://localhost:1234/v1/chat/completions';
        this.model = options.model || 'qwen2.5-7b-instruct-1m';
        this.outputDir = options.outputDir; // Required
        this.browser = null;
        this.maxContentLength = options.maxContentLength || 25000;
        this.defaultTags = options.defaultTags || ['article', 'summary', 'automated'];
    }

    async initialize() {
        if (!this.outputDir) {
            throw new Error('outputDir is required');
        }

        await fs.mkdir(this.outputDir, { recursive: true });

        this.browser = await puppeteer.launch({
          headless: 'new',
          defaultViewport: null,
          args: [
            '--no-sandbox',
            '--disable-setuid-sandbox',
            '--disable-dev-shm-usage',
            '--disable-accelerated-2d-canvas',
            '--no-first-run',
            '--no-zygote',
            '--disable-gpu',
            '--disable-blink-features=AutomationControlled',
          ],
        });

        emit('status', { message: 'Browser initialized', outputDir: this.outputDir });
    }

    async scrapeContent(url) {
        if (!this.browser) {
            throw new Error('Agent not initialized. Call initialize() first.');
        }
        const page = await this.browser.newPage();

        try {
            // Randomize user agent
            const userAgents = [
                'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
                'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
                'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36',
                'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36 Edg/119.0.0.0'
            ];
            await page.setUserAgent(userAgents[Math.floor(Math.random() * userAgents.length)]);

            // Set realistic viewport
            const viewports = [
                { width: 1920, height: 1080 },
                { width: 1366, height: 768 },
                { width: 1440, height: 900 },
                { width: 1536, height: 864 }
            ];
            const viewport = viewports[Math.floor(Math.random() * viewports.length)];
            await page.setViewport(viewport);

            // Set headers
            await page.setExtraHTTPHeaders({
                'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7',
                'Accept-Language': 'en-US,en;q=0.9',
                'Accept-Encoding': 'gzip, deflate, br',
                'Cache-Control': 'max-age=0',
                'Sec-Ch-Ua': '"Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120"',
                'Sec-Ch-Ua-Mobile': '?0',
                'Sec-Ch-Ua-Platform': '"macOS"',
                'Sec-Fetch-Dest': 'document',
                'Sec-Fetch-Mode': 'navigate',
                'Sec-Fetch-Site': 'none',
                'Sec-Fetch-User': '?1',
                'Upgrade-Insecure-Requests': '1',
            });

            // Hide automation indicators
            await page.evaluateOnNewDocument(() => {
                Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
                Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
                Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
            });

            // Random delays
            const randomDelay = () => Math.floor(Math.random() * 1000) + 500;
            const wait = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            await wait(randomDelay());

            emit('progress', { url, stage: 'navigating' });

            const response = await page.goto(url, {
                waitUntil: 'domcontentloaded',
                timeout: 25000
            });

            if (!response || !response.ok()) {
                throw new Error(`HTTP ${response?.status()} - ${response?.statusText()}`);
            }

            let finalUrl = page.url();

            // Wait for redirects (especially for search.app URLs)
            let redirectCount = 0;
            const maxRedirects = 10;

            while (finalUrl.includes('search.app') && redirectCount < maxRedirects) {
                await wait(3000);
                const newUrl = page.url();
                if (newUrl !== finalUrl) {
                    finalUrl = newUrl;
                    redirectCount++;
                    if (!finalUrl.includes('search.app')) break;
                } else {
                    redirectCount++;
                }
            }

            // Simulate human behavior
            await wait(randomDelay());
            await page.mouse.move(
                Math.floor(Math.random() * viewport.width),
                Math.floor(Math.random() * viewport.height)
            );

            // Wait for dynamic content
            try {
                await page.waitForSelector('body', { timeout: 5000 });
                await wait(2000);
            } catch (e) {
                // Continue anyway
            }

            // Extract content
            const content = await page.evaluate(() => {
                try {
                    // Remove unwanted elements
                    const elementsToRemove = document.querySelectorAll(
                        'script, style, nav, footer, aside, .ad, .advertisement, .sidebar, noscript'
                    );
                    elementsToRemove.forEach(el => el.remove());

                    // Find main content
                    const selectors = [
                        'article', '[role="main"]', 'main', '.content',
                        '.article-content', '.post-content', '.entry-content',
                        '#content', '.main-content', '.page-content'
                    ];

                    let mainContent = null;
                    let maxLength = 0;

                    for (const selector of selectors) {
                        const elements = document.querySelectorAll(selector);
                        elements.forEach(element => {
                            const text = element.innerText?.trim() || '';
                            if (text.length > maxLength && text.length > 100) {
                                mainContent = element;
                                maxLength = text.length;
                            }
                        });
                    }

                    const contentElement = mainContent || document.body;
                    const textContent = contentElement.innerText?.trim() || '';

                    return {
                        title: document.title || 'No title',
                        content: textContent,
                        url: window.location.href,
                        timestamp: new Date().toISOString(),
                        wordCount: textContent.split(/\s+/).filter(word => word.length > 0).length,
                        hasValidContent: textContent.length > 200
                    };
                } catch (evalError) {
                    return {
                        title: document.title || 'No title',
                        content: document.body?.innerText?.trim() || 'No content extracted',
                        url: window.location.href,
                        timestamp: new Date().toISOString(),
                        error: evalError.message
                    };
                }
            });

            await page.close();
            return content;

        } catch (error) {
            await page.close();
            throw new Error(`Scraping failed for ${url}: ${error.message}`);
        }
    }

    async summarizeContent(content, originalUrl) {
        try {
            let textToSummarize = content.content;
            if (textToSummarize.length > this.maxContentLength) {
                textToSummarize = textToSummarize.substring(0, this.maxContentLength) + '...';
            }

            const prompt = `You are a precise summarizer. Follow the instructions EXACTLY.
Do not add any text before or after the requested sections. Do NOT use code fences.

ARTICLE METADATA
Title: ${content.title}
URL: ${content.url}
Original URL: ${originalUrl}

ARTICLE TEXT (BEGIN)
${textToSummarize}
ARTICLE TEXT (END)

TASKS
1) Write a concise, neutral summary in Markdown for Obsidian.
   - Use 4–5 short paragraphs.
   - Then add a "Key points" bullet list (3–6 bullets).
   - Keep it factual; include the "so what" when relevant.
   - No external links, no images, no tables, no code blocks.
   - Write in the same language as the article.

2) Produce 3–5 categorization tags:
   - lowercase snake_case only (a–z, 0–9, underscore), no spaces, no dashes.
   - Examples: ai_policy, large_language_models, data_privacy

3) Provide ONE main domain/category, e.g. one of:
   technology, business, science, politics, economics, health, culture, education,
   finance, law, security, sports, entertainment, environment, general.

OUTPUT FORMAT (must match exactly — no extra sections):

SUMMARY:
[Your Markdown summary: 4–5 short paragraphs, then a "Key points" bullet list]

TAGS:
tag1, tag2, tag3, tag4, tag5

DOMAIN:
domain_one_word_from_list`;

            emit('progress', { url: originalUrl, stage: 'summarizing' });

            const response = await axios.post(this.lmStudioEndpoint, {
                model: this.model,
                messages: [{ role: 'user', content: prompt }],
                temperature: 0.7,
                max_tokens: 5500
            }, {
                timeout: 900000,
                headers: { 'Content-Type': 'application/json' }
            });

            return this.parseLLMResponse(response.data.choices[0].message.content);

        } catch (error) {
            throw new Error(`Failed to get summary from LLM: ${error.message}`);
        }
    }

    parseLLMResponse(response) {
        const sections = { summary: '', tags: [], domain: 'general' };
        const lines = response.split('\n');
        let currentSection = '';

        for (const line of lines) {
            const trimmedLine = line.trim();

            if (trimmedLine.toUpperCase().includes('SUMMARY:')) {
                currentSection = 'summary';
                continue;
            } else if (trimmedLine.toUpperCase().includes('TAGS:')) {
                currentSection = 'tags';
                continue;
            } else if (trimmedLine.toUpperCase().includes('DOMAIN:')) {
                currentSection = 'domain';
                continue;
            }

            if (currentSection === 'summary' && trimmedLine) {
                sections.summary += (sections.summary ? '\n' : '') + trimmedLine;
            } else if (currentSection === 'tags' && trimmedLine) {
                const tagText = trimmedLine.replace(/[\[\]]/g, '');
                sections.tags = tagText.split(',').map(tag => tag.trim().toLowerCase()).filter(tag => tag);
                break;
            } else if (currentSection === 'domain' && trimmedLine) {
                sections.domain = trimmedLine.toLowerCase();
                break;
            }
        }

        if (!sections.summary) {
            sections.summary = response;
        }

        return sections;
    }

    sanitizeFilename(text) {
        return text
            .replace(/[^\w\s-]/g, '')
            .replace(/\s+/g, '-')
            .toLowerCase()
            .substring(0, 50);
    }

    generateObsidianNote(content, originalUrl, llmResponse, index) {
        const now = new Date();
        const dateStr = now.toISOString().split('T')[0];
        const timeStr = now.toTimeString().split(' ')[0];

        const titleSlug = this.sanitizeFilename(content.title || 'untitled');
        const filename = `${dateStr}-${titleSlug}.md`;

        const allTags = [...new Set([...this.defaultTags, ...llmResponse.tags])];

        let siteName = '';
        try {
            const urlObj = new URL(content.url);
            siteName = urlObj.hostname.replace('www.', '');
        } catch (e) {
            siteName = 'unknown';
        }

        const frontmatter = `---
title: "${content.title.replace(/"/g, '\\"')}"
date: ${dateStr}
time: ${timeStr}
url: "${content.url}"
original_url: "${originalUrl}"
site: "${siteName}"
domain: "${llmResponse.domain}"
content_length: ${content.content.length}
status: "summarized"
tags: [${allTags.map(tag => `"${tag}"`).join(', ')}]
---`;

        const noteContent = `${frontmatter}

# ${content.title}

## Summary

${llmResponse.summary}

## Source Information

- **Original URL**: ${originalUrl}
- **Final URL**: [${content.url}](${content.url})
- **Site**: ${siteName}
- **Domain**: ${llmResponse.domain}
- **Processed**: ${dateStr} at ${timeStr}
- **Content Length**: ${content.content.length.toLocaleString()} characters

## Tags

${allTags.map(tag => `#${tag.replace(/\s+/g, '-')}`).join(' ')}

---
*This summary was automatically generated using local LLM processing.*`;

        return {
            filename,
            content: noteContent,
            metadata: {
                title: content.title,
                url: content.url,
                originalUrl: originalUrl,
                tags: allTags,
                domain: llmResponse.domain,
                site: siteName,
                dateProcessed: dateStr,
                contentLength: content.content.length
            }
        };
    }

    async processLink(url, index = 0) {
        try {
            emit('progress', { url, index, stage: 'starting' });

            const content = await this.scrapeContent(url);

            if (!content.content || content.content.length < 100) {
                throw new Error('Content too short or empty');
            }

            const llmResponse = await this.summarizeContent(content, url);
            const obsidianNote = this.generateObsidianNote(content, url, llmResponse, index);

            const filepath = path.join(this.outputDir, obsidianNote.filename);
            await fs.writeFile(filepath, obsidianNote.content, 'utf-8');

            emit('success', {
                url,
                index,
                filepath,
                title: content.title,
                filename: obsidianNote.filename
            });

            return {
                originalUrl: url,
                finalUrl: content.url,
                title: content.title,
                summary: llmResponse.summary,
                tags: llmResponse.tags,
                domain: llmResponse.domain,
                contentLength: content.content.length,
                timestamp: content.timestamp,
                filename: obsidianNote.filename,
                filepath: filepath,
                success: true
            };

        } catch (error) {
            emit('error', { url, index, error: error.message });
            return {
                originalUrl: url,
                error: error.message,
                success: false,
                timestamp: new Date().toISOString()
            };
        }
    }

    async processLinks(links) {
        const results = [];

        for (let i = 0; i < links.length; i++) {
            const result = await this.processLink(links[i], i);
            results.push(result);

            if (i < links.length - 1) {
                await new Promise(resolve => setTimeout(resolve, 2000));
            }
        }

        const summaryReport = {
            totalLinks: links.length,
            successfulSummaries: results.filter(r => r.success).length,
            failedSummaries: results.filter(r => !r.success).length,
            results: results,
            generatedAt: new Date().toISOString(),
            outputDirectory: this.outputDir
        };

        emit('complete', summaryReport);
        return summaryReport;
    }

    async cleanup() {
        if (this.browser) {
            await this.browser.close();
        }
    }
}

// CLI Interface
async function main() {
    const args = process.argv.slice(2);

    if (args.length === 0 || args.includes('--help')) {
        emit('error', { error: 'No arguments provided. Use --help for usage.' });
        console.error(`
Link Summarizer - NeuroFlow Integration

Usage:
  node link-summarizer.js --output-dir <path> <url1> [url2] ...
  node link-summarizer.js --output-dir <path> --file <links.txt>

Required:
  --output-dir <path>    Directory to save summary notes

Options:
  --endpoint <url>       LM Studio endpoint (default: http://localhost:1234/v1/chat/completions)
  --model <name>         Model name (default: openai/gpt-oss-20b)
  --max-content <n>      Max content length for LLM (default: 25000)
  --tags <tags>          Default tags, comma-separated
  --file <path>          Read URLs from file (one per line)
        `);
        process.exit(1);
    }

    // Parse arguments
    const options = {};
    const links = [];

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];

        switch (arg) {
            case '--endpoint':
                options.lmStudioEndpoint = args[++i];
                break;
            case '--model':
                options.model = args[++i];
                break;
            case '--output-dir':
                options.outputDir = args[++i];
                break;
            case '--max-content':
                options.maxContentLength = parseInt(args[++i]);
                break;
            case '--tags':
                options.defaultTags = args[++i].split(',').map(tag => tag.trim());
                break;
            case '--file':
                const filePath = args[++i];
                try {
                    const fileContent = await fs.readFile(filePath, 'utf-8');
                    const fileLinks = fileContent.split('\n')
                        .map(line => line.trim())
                        .filter(line => line && !line.startsWith('#'));
                    links.push(...fileLinks);
                } catch (error) {
                    emit('error', { error: `Failed to read file ${filePath}: ${error.message}` });
                    process.exit(1);
                }
                break;
            default:
                if (arg.startsWith('http')) {
                    links.push(arg);
                }
                break;
        }
    }

    if (!options.outputDir) {
        emit('error', { error: '--output-dir is required' });
        process.exit(1);
    }

    if (links.length === 0) {
        emit('error', { error: 'No valid links provided' });
        process.exit(1);
    }

    const agent = new LinkSummarizerAgent(options);

    try {
        await agent.initialize();
        const results = await agent.processLinks(links);

        if (results.failedSummaries > 0) {
            process.exit(1);
        }

    } catch (error) {
        emit('error', { error: error.message });
        process.exit(1);
    } finally {
        await agent.cleanup();
    }
}

module.exports = LinkSummarizerAgent;

if (require.main === module) {
    main().catch(err => {
        emit('error', { error: err.message });
        process.exit(1);
    });
}
