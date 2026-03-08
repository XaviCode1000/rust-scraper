---
title: Gemini CLI documentation
url: https://geminicli.com/docs/
date: 2026-03-08
excerpt: |-
  Gemini CLI brings the power of Gemini models directly into your terminal. Use it
  to understand code, automate tasks, and build workflows with your local project
  context.---

---
canonical: <https://geminicli.com/docs/>
meta-generator: Starlight v0.36.1
meta-og:image: <https://geminicli.com/assets/social-poster.png>
meta-og:locale: en
meta-og:site_name: Gemini CLI
meta-og:title: Gemini CLI documentation
meta-og:type: article
meta-og:url: <https://geminicli.com/docs/>
meta-twitter:card: summary_large_image
meta-twitter:image: <https://geminicli.com/assets/social-poster.png>
meta-viewport: width=device-width, initial-scale=1
title: Gemini CLI documentation | Gemini CLI
---

  [Skip to content](#_top)

 [![Gemini CLI Icon](/_astro/icon.Bo4M5sF3.png) Gemini CLI](/)

 [Plans](/plans/) [Home](/)

 [Extensions](/extensions/)

 [Gallery](/extensions/) [About Extensions](/extensions/about)

 [Docs](/docs/) [Reference](/docs/reference/commands) [Resources](/docs/resources/quota-and-pricing) [Changelog](/docs/changelogs/)

   ![SVG Image](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIGZpbGw9ImN1cnJlbnRDb2xvciIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxZW07IiBhcmlhLWhpZGRlbj0idHJ1ZSIgaGVpZ2h0PSIxNiIgdmlld0JveD0iMCAwIDI0IDI0IiBjbGFzcz0iYXN0cm8tdjM3bW5rbnogYXN0cm8tYzZ2c29xYXMiPjxwYXRoIGQ9Ik0yMS43MSAyMC4yOSAxOCAxNi42MUE5IDkgMCAxIDAgMTYuNjEgMThsMy42OCAzLjY4YS45OTkuOTk5IDAgMCAwIDEuNDIgMCAxIDEgMCAwIDAgMC0xLjM5Wk0xMSAxOGE3IDcgMCAxIDEgMC0xNCA3IDcgMCAwIDEgMCAxNFoiIC8+PC9zdmc+) Search  ``Ctrl``K``

  Cancel

 [![SVG Image](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjRweCIgYXJpYS1oaWRkZW49InRydWUiIGhlaWdodD0iMjRweCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJjdXJyZW50Q29sb3IiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgY2xhc3M9ImZlZWRiYWNrLWljb24gYXN0cm8tM2VmNmtzcjIiPiA8cGF0aCBkPSJNMCAwaDI0djI0SDBWMHoiIGZpbGw9Im5vbmUiIGNsYXNzPSJhc3Ryby0zZWY2a3NyMiIgLz4gPHBhdGggZD0iTTIwIDJINGMtMS4xIDAtMiAuOS0yIDJ2MThsNC00aDE0YzEuMSAwIDItLjkgMi0yVjRjMC0xLjEtLjktMi0yLTJ6bTAgMTRINmwtMiAyVjRoMTZ2MTJ6IiBjbGFzcz0iYXN0cm8tM2VmNmtzcjIiIC8+IDxwYXRoIGQ9Ik0xMSA1aDJ2NmgtMnptMCA4aDJ2MmgtMnoiIGNsYXNzPSJhc3Ryby0zZWY2a3NyMiIgLz4gPC9zdmc+) Feedback](https://github.com/google-gemini/gemini-cli/issues/new) [GitHub ![SVG Image](data:image/svg+xml;base64,PHN2ZyBmaWxsPSJjdXJyZW50Q29sb3IiIGFyaWEtaGlkZGVuPSJ0cnVlIiB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiIgY2xhc3M9ImFzdHJvLTNlZjZrc3IyIj4gPHBhdGggZD0iTTEyIC4zYTEyIDEyIDAgMCAwLTMuOCAyMy4zOGMuNi4xMi44My0uMjYuODMtLjU3TDkgMjEuMDdjLTMuMzQuNzItNC4wNC0xLjYxLTQuMDQtMS42MS0uNTUtMS4zOS0xLjM0LTEuNzYtMS4zNC0xLjc2LTEuMDgtLjc0LjA5LS43My4wOS0uNzMgMS4yLjA5IDEuODMgMS4yNCAxLjgzIDEuMjQgMS4wOCAxLjgzIDIuODEgMS4zIDMuNSAxIC4xLS43OC40Mi0xLjMxLjc2LTEuNjEtMi42Ny0uMy01LjQ3LTEuMzMtNS40Ny01LjkzIDAtMS4zMS40Ny0yLjM4IDEuMjQtMy4yMi0uMTQtLjMtLjU0LTEuNTIuMS0zLjE4IDAgMCAxLS4zMiAzLjMgMS4yM2ExMS41IDExLjUgMCAwIDEgNiAwYzIuMjgtMS41NSAzLjI5LTEuMjMgMy4yOS0xLjIzLjY0IDEuNjYuMjQgMi44OC4xMiAzLjE4YTQuNjUgNC42NSAwIDAgMSAxLjIzIDMuMjJjMCA0LjYxLTIuOCA1LjYzLTUuNDggNS45Mi40Mi4zNi44MSAxLjEuODEgMi4yMmwtLjAxIDMuMjljMCAuMzEuMi42OS44Mi41N0ExMiAxMiAwIDAgMCAxMiAuM1oiIGNsYXNzPSJhc3Ryby0zZWY2a3NyMiIgLz4gPC9zdmc+) GitHub](https://github.com/google-gemini/gemini-cli)

   Select theme ![SVG Image](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIGFyaWEtaGlkZGVuPSJ0cnVlIiBoZWlnaHQ9IjE2IiBmaWxsPSJjdXJyZW50Q29sb3IiIHZpZXdCb3g9IjAgMCAyNCAyNCIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxZW07IiBjbGFzcz0iaWNvbiBsYWJlbC1pY29uIGFzdHJvLTR5cGh0b2VuIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJNMjEgMTRoLTFWN2EzIDMgMCAwIDAtMy0zSDdhMyAzIDAgMCAwLTMgM3Y3SDNhMSAxIDAgMCAwLTEgMXYyYTMgMyAwIDAgMCAzIDNoMTRhMyAzIDAgMCAwIDMtM3YtMmExIDEgMCAwIDAtMS0xWk02IDdhMSAxIDAgMCAxIDEtMWgxMGExIDEgMCAwIDEgMSAxdjdINlY3Wm0xNCAxMGExIDEgMCAwIDEtMSAxSDVhMSAxIDAgMCAxLTEtMXYtMWgxNnYxWiIgLz48L3N2Zz4=)  Dark
Light
- Auto

 ![SVG Image](data:image/svg+xml;base64,PHN2ZyBmaWxsPSJjdXJyZW50Q29sb3IiIGFyaWEtaGlkZGVuPSJ0cnVlIiB2aWV3Qm94PSIwIDAgMjQgMjQiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMWVtOyIgaGVpZ2h0PSIxNiIgd2lkdGg9IjE2IiBjbGFzcz0iaWNvbiBjYXJldCBhc3Ryby00eXBodG9lbiBhc3Ryby1jNnZzb3FhcyI+PHBhdGggZD0iTTE3IDkuMTdhMSAxIDAgMCAwLTEuNDEgMEwxMiAxMi43MSA4LjQ2IDkuMTdhMSAxIDAgMSAwLTEuNDEgMS40Mmw0LjI0IDQuMjRhMS4wMDIgMS4wMDIgMCAwIDAgMS40MiAwTDE3IDEwLjU5YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)

       ![SVG Image](data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiIgZmlsbD0iY3VycmVudENvbG9yIiBhcmlhLWhpZGRlbj0idHJ1ZSIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxZW07IiBoZWlnaHQ9IjE2IiBjbGFzcz0ib3Blbi1tZW51IGFzdHJvLWppZjczeXp3IGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJNMyA4aDE4YTEgMSAwIDEgMCAwLTJIM2ExIDEgMCAwIDAgMCAyWm0xOCA4SDNhMSAxIDAgMCAwIDAgMmgxOGExIDEgMCAwIDAgMC0yWm0wLTVIM2ExIDEgMCAwIDAgMCAyaDE4YTEgMSAwIDAgMCAwLTJaIiAvPjwvc3ZnPg==) ![SVG Image](data:image/svg+xml;base64,PHN2ZyBhcmlhLWhpZGRlbj0idHJ1ZSIgd2lkdGg9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMWVtOyIgZmlsbD0iY3VycmVudENvbG9yIiBoZWlnaHQ9IjE2IiBjbGFzcz0iY2xvc2UtbWVudSBhc3Ryby1qaWY3M3l6dyBhc3Ryby1jNnZzb3FhcyI+PHBhdGggZD0ibTEzLjQxIDEyIDYuMy02LjI5YTEuMDA0IDEuMDA0IDAgMSAwLTEuNDItMS40MkwxMiAxMC41OWwtNi4yOS02LjNhMS4wMDQgMS4wMDQgMCAwIDAtMS40MiAxLjQybDYuMyA2LjI5LTYuMyA2LjI5YTEgMSAwIDAgMCAwIDEuNDIuOTk4Ljk5OCAwIDAgMCAxLjQyIDBsNi4yOS02LjMgNi4yOSA2LjNhLjk5OS45OTkgMCAwIDAgMS40MiAwIDEgMSAwIDAgMCAwLTEuNDJMMTMuNDEgMTJaIiAvPjwvc3ZnPg==)

-  

**Get started   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBzdHlsZT0iLS1zbC1pY29uLXNpemU6IDEuMjVyZW07IiB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiIgYXJpYS1oaWRkZW49InRydWUiIGhlaWdodD0iMTYiIGZpbGw9ImN1cnJlbnRDb2xvciIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

- [Overview](/docs/)
- [Quickstart](/docs/get-started/)
- [Installation](/docs/get-started/installation/)
- [Authentication](/docs/get-started/authentication/)
- [Examples](/docs/get-started/examples/)
- [CLI cheatsheet](/docs/cli/cli-reference/)
- [Gemini 3 on Gemini CLI](/docs/get-started/gemini-3/)

-  

**Use Gemini CLI   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBmaWxsPSJjdXJyZW50Q29sb3IiIHZpZXdCb3g9IjAgMCAyNCAyNCIgYXJpYS1oaWRkZW49InRydWUiIGhlaWdodD0iMTYiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMS4yNXJlbTsiIHdpZHRoPSIxNiIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

- [File management](/docs/cli/tutorials/file-management/)
- [Get started with Agent skills](/docs/cli/tutorials/skills-getting-started/)
- [Manage context and memory](/docs/cli/tutorials/memory-management/)
- [Execute shell commands](/docs/cli/tutorials/shell-commands/)
- [Manage sessions and history](/docs/cli/tutorials/session-management/)
- [Plan tasks with todos](/docs/cli/tutorials/task-planning/)
- [Web search and fetch](/docs/cli/tutorials/web-tools/)
- [Set up an MCP server](/docs/cli/tutorials/mcp-setup/)
- [Automate tasks](/docs/cli/tutorials/automation/)

-  

**Features   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiBhcmlhLWhpZGRlbj0idHJ1ZSIgdmlld0JveD0iMCAwIDI0IDI0IiB3aWR0aD0iMTYiIGZpbGw9ImN1cnJlbnRDb2xvciIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxLjI1cmVtOyIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

-  

**Extensions   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBzdHlsZT0iLS1zbC1pY29uLXNpemU6IDEuMjVyZW07IiBhcmlhLWhpZGRlbj0idHJ1ZSIgd2lkdGg9IjE2IiBoZWlnaHQ9IjE2IiBmaWxsPSJjdXJyZW50Q29sb3IiIHZpZXdCb3g9IjAgMCAyNCAyNCIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

    -  [Overview](/docs/extensions/)
    -  [User guide: Install and manage](/docs/extensions/#manage-extensions)
    -  [Developer guide: Build extensions](/docs/extensions/writing-extensions/)
    -  [Developer guide: Best practices](/docs/extensions/best-practices/)
    -  [Developer guide: Releasing](/docs/extensions/releasing/)
    -  [Developer guide: Reference](/docs/extensions/reference/)

- [Agent Skills](/docs/cli/skills/)
- [Checkpointing](/docs/cli/checkpointing/)
- [Headless mode](/docs/cli/headless/)
-  

**Hooks   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBhcmlhLWhpZGRlbj0idHJ1ZSIgd2lkdGg9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMS4yNXJlbTsiIGZpbGw9ImN1cnJlbnRDb2xvciIgaGVpZ2h0PSIxNiIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

    -  [Overview](/docs/hooks/)
    -  [Reference](/docs/hooks/reference/)

- [IDE integration](/docs/ide-integration/)
- [MCP servers](/docs/tools/mcp-server/)
- [Model routing](/docs/cli/model-routing/)
- [Model selection](/docs/cli/model/)
- [Plan mode 🔬](/docs/cli/plan-mode/)
- [Subagents 🔬](/docs/core/subagents/)
- [Remote subagents 🔬](/docs/core/remote-agents/)
- [Rewind](/docs/cli/rewind/)
- [Sandboxing](/docs/cli/sandbox/)
- [Settings](/docs/cli/settings/)
- [Telemetry](/docs/cli/telemetry/)
- [Token caching](/docs/cli/token-caching/)

-  

**Configuration   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBmaWxsPSJjdXJyZW50Q29sb3IiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMS4yNXJlbTsiIHdpZHRoPSIxNiIgdmlld0JveD0iMCAwIDI0IDI0IiBhcmlhLWhpZGRlbj0idHJ1ZSIgaGVpZ2h0PSIxNiIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

- [Custom commands](/docs/cli/custom-commands/)
- [Enterprise configuration](/docs/cli/enterprise/)
- [Ignore files (.geminiignore)](/docs/cli/gemini-ignore/)
- [Model configuration](/docs/cli/generation-settings/)
- [Project context (GEMINI.md)](/docs/cli/gemini-md/)
- [Settings](/docs/cli/settings/)
- [System prompt override](/docs/cli/system-prompt/)
- [Themes](/docs/cli/themes/)
- [Trusted folders](/docs/cli/trusted-folders/)

-  

**Development   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBzdHlsZT0iLS1zbC1pY29uLXNpemU6IDEuMjVyZW07IiBmaWxsPSJjdXJyZW50Q29sb3IiIHdpZHRoPSIxNiIgYXJpYS1oaWRkZW49InRydWUiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAyNCAyNCIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

- [Contribution guide](/docs/contributing/)
- [Integration testing](/docs/integration-tests/)
- [Issue and PR automation](/docs/issue-and-pr-automation/)
- [Local development](/docs/local-development/)
- [NPM package structure](/docs/npm/)

 [GitHub![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIGFyaWEtaGlkZGVuPSJ0cnVlIiB3aWR0aD0iMTYiIGZpbGw9ImN1cnJlbnRDb2xvciIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxZW07IiBjbGFzcz0iYXN0cm8td3k0dGU2Z2EgYXN0cm8tYzZ2c29xYXMiPjxwYXRoIGQ9Ik0xMiAuM2ExMiAxMiAwIDAgMC0zLjggMjMuMzhjLjYuMTIuODMtLjI2LjgzLS41N0w5IDIxLjA3Yy0zLjM0LjcyLTQuMDQtMS42MS00LjA0LTEuNjEtLjU1LTEuMzktMS4zNC0xLjc2LTEuMzQtMS43Ni0xLjA4LS43NC4wOS0uNzMuMDktLjczIDEuMi4wOSAxLjgzIDEuMjQgMS44MyAxLjI0IDEuMDggMS44MyAyLjgxIDEuMyAzLjUgMSAuMS0uNzguNDItMS4zMS43Ni0xLjYxLTIuNjctLjMtNS40Ny0xLjMzLTUuNDctNS45MyAwLTEuMzEuNDctMi4zOCAxLjI0LTMuMjItLjE0LS4zLS41NC0xLjUyLjEtMy4xOCAwIDAgMS0uMzIgMy4zIDEuMjNhMTEuNSAxMS41IDAgMCAxIDYgMGMyLjI4LTEuNTUgMy4yOS0xLjIzIDMuMjktMS4yMy42NCAxLjY2LjI0IDIuODguMTIgMy4xOGE0LjY1IDQuNjUgMCAwIDEgMS4yMyAzLjIyYzAgNC42MS0yLjggNS42My01LjQ4IDUuOTIuNDIuMzYuODEgMS4xLjgxIDIuMjJsLS4wMSAzLjI5YzAgLjMxLjIuNjkuODIuNTdBMTIgMTIgMCAwIDAgMTIgLjNaIiAvPjwvc3ZnPg==)](https://github.com/google-gemini/gemini-cli)

   Select theme ![SVG Image](data:image/svg+xml;base64,PHN2ZyBmaWxsPSJjdXJyZW50Q29sb3IiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMWVtOyIgd2lkdGg9IjE2IiBoZWlnaHQ9IjE2IiBhcmlhLWhpZGRlbj0idHJ1ZSIgdmlld0JveD0iMCAwIDI0IDI0IiBjbGFzcz0iaWNvbiBsYWJlbC1pY29uIGFzdHJvLTR5cGh0b2VuIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJNMjEgMTRoLTFWN2EzIDMgMCAwIDAtMy0zSDdhMyAzIDAgMCAwLTMgM3Y3SDNhMSAxIDAgMCAwLTEgMXYyYTMgMyAwIDAgMCAzIDNoMTRhMyAzIDAgMCAwIDMtM3YtMmExIDEgMCAwIDAtMS0xWk02IDdhMSAxIDAgMCAxIDEtMWgxMGExIDEgMCAwIDEgMSAxdjdINlY3Wm0xNCAxMGExIDEgMCAwIDEtMSAxSDVhMSAxIDAgMCAxLTEtMXYtMWgxNnYxWiIgLz48L3N2Zz4=)  Dark
Light
- Auto

 ![SVG Image](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIGFyaWEtaGlkZGVuPSJ0cnVlIiBoZWlnaHQ9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIGZpbGw9ImN1cnJlbnRDb2xvciIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxZW07IiBjbGFzcz0iaWNvbiBjYXJldCBhc3Ryby00eXBodG9lbiBhc3Ryby1jNnZzb3FhcyI+PHBhdGggZD0iTTE3IDkuMTdhMSAxIDAgMCAwLTEuNDEgMEwxMiAxMi43MSA4LjQ2IDkuMTdhMSAxIDAgMSAwLTEuNDEgMS40Mmw0LjI0IDQuMjRhMS4wMDIgMS4wMDIgMCAwIDAgMS40MiAwTDE3IDEwLjU5YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)

-  

**Reference   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBzdHlsZT0iLS1zbC1pY29uLXNpemU6IDEuMjVyZW07IiBoZWlnaHQ9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiIgZmlsbD0iY3VycmVudENvbG9yIiBhcmlhLWhpZGRlbj0idHJ1ZSIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

- [Command reference](/docs/reference/commands/)
- [Configuration reference](/docs/reference/configuration/)
- [Keyboard shortcuts](/docs/reference/keyboard-shortcuts/)
- [Memory import processor](/docs/reference/memport/)
- [Policy engine](/docs/reference/policy-engine/)
- [Tools reference](/docs/reference/tools/)

 [GitHub![SVG Image](data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMjQgMjQiIGFyaWEtaGlkZGVuPSJ0cnVlIiBzdHlsZT0iLS1zbC1pY29uLXNpemU6IDFlbTsiIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiIgZmlsbD0iY3VycmVudENvbG9yIiBjbGFzcz0iYXN0cm8td3k0dGU2Z2EgYXN0cm8tYzZ2c29xYXMiPjxwYXRoIGQ9Ik0xMiAuM2ExMiAxMiAwIDAgMC0zLjggMjMuMzhjLjYuMTIuODMtLjI2LjgzLS41N0w5IDIxLjA3Yy0zLjM0LjcyLTQuMDQtMS42MS00LjA0LTEuNjEtLjU1LTEuMzktMS4zNC0xLjc2LTEuMzQtMS43Ni0xLjA4LS43NC4wOS0uNzMuMDktLjczIDEuMi4wOSAxLjgzIDEuMjQgMS44MyAxLjI0IDEuMDggMS44MyAyLjgxIDEuMyAzLjUgMSAuMS0uNzguNDItMS4zMS43Ni0xLjYxLTIuNjctLjMtNS40Ny0xLjMzLTUuNDctNS45MyAwLTEuMzEuNDctMi4zOCAxLjI0LTMuMjItLjE0LS4zLS41NC0xLjUyLjEtMy4xOCAwIDAgMS0uMzIgMy4zIDEuMjNhMTEuNSAxMS41IDAgMCAxIDYgMGMyLjI4LTEuNTUgMy4yOS0xLjIzIDMuMjktMS4yMy42NCAxLjY2LjI0IDIuODguMTIgMy4xOGE0LjY1IDQuNjUgMCAwIDEgMS4yMyAzLjIyYzAgNC42MS0yLjggNS42My01LjQ4IDUuOTIuNDIuMzYuODEgMS4xLjgxIDIuMjJsLS4wMSAzLjI5YzAgLjMxLjIuNjkuODIuNTdBMTIgMTIgMCAwIDAgMTIgLjNaIiAvPjwvc3ZnPg==)](https://github.com/google-gemini/gemini-cli)

   Select theme ![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMWVtOyIgZmlsbD0iY3VycmVudENvbG9yIiBhcmlhLWhpZGRlbj0idHJ1ZSIgd2lkdGg9IjE2IiBjbGFzcz0iaWNvbiBsYWJlbC1pY29uIGFzdHJvLTR5cGh0b2VuIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJNMjEgMTRoLTFWN2EzIDMgMCAwIDAtMy0zSDdhMyAzIDAgMCAwLTMgM3Y3SDNhMSAxIDAgMCAwLTEgMXYyYTMgMyAwIDAgMCAzIDNoMTRhMyAzIDAgMCAwIDMtM3YtMmExIDEgMCAwIDAtMS0xWk02IDdhMSAxIDAgMCAxIDEtMWgxMGExIDEgMCAwIDEgMSAxdjdINlY3Wm0xNCAxMGExIDEgMCAwIDEtMSAxSDVhMSAxIDAgMCAxLTEtMXYtMWgxNnYxWiIgLz48L3N2Zz4=)  Dark
Light
- Auto

 ![SVG Image](data:image/svg+xml;base64,PHN2ZyBhcmlhLWhpZGRlbj0idHJ1ZSIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJjdXJyZW50Q29sb3IiIGhlaWdodD0iMTYiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMWVtOyIgd2lkdGg9IjE2IiBjbGFzcz0iaWNvbiBjYXJldCBhc3Ryby00eXBodG9lbiBhc3Ryby1jNnZzb3FhcyI+PHBhdGggZD0iTTE3IDkuMTdhMSAxIDAgMCAwLTEuNDEgMEwxMiAxMi43MSA4LjQ2IDkuMTdhMSAxIDAgMSAwLTEuNDEgMS40Mmw0LjI0IDQuMjRhMS4wMDIgMS4wMDIgMCAwIDAgMS40MiAwTDE3IDEwLjU5YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)

-  

**Resources   ![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiBhcmlhLWhpZGRlbj0idHJ1ZSIgd2lkdGg9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIGZpbGw9ImN1cnJlbnRDb2xvciIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxLjI1cmVtOyIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

- [FAQ](/docs/resources/faq/)
- [Quota and pricing](/docs/resources/quota-and-pricing/)
- [Terms and privacy](/docs/resources/tos-privacy/)
- [Troubleshooting](/docs/resources/troubleshooting/)
- [Uninstall](/docs/resources/uninstall/)

 [GitHub![SVG Image](data:image/svg+xml;base64,PHN2ZyBzdHlsZT0iLS1zbC1pY29uLXNpemU6IDFlbTsiIHdpZHRoPSIxNiIgYXJpYS1oaWRkZW49InRydWUiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0iY3VycmVudENvbG9yIiBjbGFzcz0iYXN0cm8td3k0dGU2Z2EgYXN0cm8tYzZ2c29xYXMiPjxwYXRoIGQ9Ik0xMiAuM2ExMiAxMiAwIDAgMC0zLjggMjMuMzhjLjYuMTIuODMtLjI2LjgzLS41N0w5IDIxLjA3Yy0zLjM0LjcyLTQuMDQtMS42MS00LjA0LTEuNjEtLjU1LTEuMzktMS4zNC0xLjc2LTEuMzQtMS43Ni0xLjA4LS43NC4wOS0uNzMuMDktLjczIDEuMi4wOSAxLjgzIDEuMjQgMS44MyAxLjI0IDEuMDggMS44MyAyLjgxIDEuMyAzLjUgMSAuMS0uNzguNDItMS4zMS43Ni0xLjYxLTIuNjctLjMtNS40Ny0xLjMzLTUuNDctNS45MyAwLTEuMzEuNDctMi4zOCAxLjI0LTMuMjItLjE0LS4zLS41NC0xLjUyLjEtMy4xOCAwIDAgMS0uMzIgMy4zIDEuMjNhMTEuNSAxMS41IDAgMCAxIDYgMGMyLjI4LTEuNTUgMy4yOS0xLjIzIDMuMjktMS4yMy42NCAxLjY2LjI0IDIuODguMTIgMy4xOGE0LjY1IDQuNjUgMCAwIDEgMS4yMyAzLjIyYzAgNC42MS0yLjggNS42My01LjQ4IDUuOTIuNDIuMzYuODEgMS4xLjgxIDIuMjJsLS4wMSAzLjI5YzAgLjMxLjIuNjkuODIuNTdBMTIgMTIgMCAwIDAgMTIgLjNaIiAvPjwvc3ZnPg==)](https://github.com/google-gemini/gemini-cli)

   Select theme ![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB3aWR0aD0iMTYiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0iY3VycmVudENvbG9yIiBhcmlhLWhpZGRlbj0idHJ1ZSIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxZW07IiBjbGFzcz0iaWNvbiBsYWJlbC1pY29uIGFzdHJvLTR5cGh0b2VuIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJNMjEgMTRoLTFWN2EzIDMgMCAwIDAtMy0zSDdhMyAzIDAgMCAwLTMgM3Y3SDNhMSAxIDAgMCAwLTEgMXYyYTMgMyAwIDAgMCAzIDNoMTRhMyAzIDAgMCAwIDMtM3YtMmExIDEgMCAwIDAtMS0xWk02IDdhMSAxIDAgMCAxIDEtMWgxMGExIDEgMCAwIDEgMSAxdjdINlY3Wm0xNCAxMGExIDEgMCAwIDEtMSAxSDVhMSAxIDAgMCAxLTEtMXYtMWgxNnYxWiIgLz48L3N2Zz4=)  Dark
Light
- Auto

 ![SVG Image](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMWVtOyIgZmlsbD0iY3VycmVudENvbG9yIiBoZWlnaHQ9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIGFyaWEtaGlkZGVuPSJ0cnVlIiBjbGFzcz0iaWNvbiBjYXJldCBhc3Ryby00eXBodG9lbiBhc3Ryby1jNnZzb3FhcyI+PHBhdGggZD0iTTE3IDkuMTdhMSAxIDAgMCAwLTEuNDEgMEwxMiAxMi43MSA4LjQ2IDkuMTdhMSAxIDAgMSAwLTEuNDEgMS40Mmw0LjI0IDQuMjRhMS4wMDIgMS4wMDIgMCAwIDAgMS40MiAwTDE3IDEwLjU5YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)

-  

**Releases   ![SVG Image](data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMjQgMjQiIGZpbGw9ImN1cnJlbnRDb2xvciIgd2lkdGg9IjE2IiBzdHlsZT0iLS1zbC1pY29uLXNpemU6IDEuMjVyZW07IiBhcmlhLWhpZGRlbj0idHJ1ZSIgaGVpZ2h0PSIxNiIgY2xhc3M9ImNhcmV0IGFzdHJvLTNpaTd4eG1zIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

- [Release notes](/docs/changelogs/)
- [Stable release](/docs/changelogs/latest/)
- [Preview release](/docs/changelogs/preview/)

 [GitHub![SVG Image](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIGFyaWEtaGlkZGVuPSJ0cnVlIiBmaWxsPSJjdXJyZW50Q29sb3IiIGhlaWdodD0iMTYiIHN0eWxlPSItLXNsLWljb24tc2l6ZTogMWVtOyIgdmlld0JveD0iMCAwIDI0IDI0IiBjbGFzcz0iYXN0cm8td3k0dGU2Z2EgYXN0cm8tYzZ2c29xYXMiPjxwYXRoIGQ9Ik0xMiAuM2ExMiAxMiAwIDAgMC0zLjggMjMuMzhjLjYuMTIuODMtLjI2LjgzLS41N0w5IDIxLjA3Yy0zLjM0LjcyLTQuMDQtMS42MS00LjA0LTEuNjEtLjU1LTEuMzktMS4zNC0xLjc2LTEuMzQtMS43Ni0xLjA4LS43NC4wOS0uNzMuMDktLjczIDEuMi4wOSAxLjgzIDEuMjQgMS44MyAxLjI0IDEuMDggMS44MyAyLjgxIDEuMyAzLjUgMSAuMS0uNzguNDItMS4zMS43Ni0xLjYxLTIuNjctLjMtNS40Ny0xLjMzLTUuNDctNS45MyAwLTEuMzEuNDctMi4zOCAxLjI0LTMuMjItLjE0LS4zLS41NC0xLjUyLjEtMy4xOCAwIDAgMS0uMzIgMy4zIDEuMjNhMTEuNSAxMS41IDAgMCAxIDYgMGMyLjI4LTEuNTUgMy4yOS0xLjIzIDMuMjktMS4yMy42NCAxLjY2LjI0IDIuODguMTIgMy4xOGE0LjY1IDQuNjUgMCAwIDEgMS4yMyAzLjIyYzAgNC42MS0yLjggNS42My01LjQ4IDUuOTIuNDIuMzYuODEgMS4xLjgxIDIuMjJsLS4wMSAzLjI5YzAgLjMxLjIuNjkuODIuNTdBMTIgMTIgMCAwIDAgMTIgLjNaIiAvPjwvc3ZnPg==)](https://github.com/google-gemini/gemini-cli)

   Select theme ![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxZW07IiBhcmlhLWhpZGRlbj0idHJ1ZSIgZmlsbD0iY3VycmVudENvbG9yIiBjbGFzcz0iaWNvbiBsYWJlbC1pY29uIGFzdHJvLTR5cGh0b2VuIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJNMjEgMTRoLTFWN2EzIDMgMCAwIDAtMy0zSDdhMyAzIDAgMCAwLTMgM3Y3SDNhMSAxIDAgMCAwLTEgMXYyYTMgMyAwIDAgMCAzIDNoMTRhMyAzIDAgMCAwIDMtM3YtMmExIDEgMCAwIDAtMS0xWk02IDdhMSAxIDAgMCAxIDEtMWgxMGExIDEgMCAwIDEgMSAxdjdINlY3Wm0xNCAxMGExIDEgMCAwIDEtMSAxSDVhMSAxIDAgMCAxLTEtMXYtMWgxNnYxWiIgLz48L3N2Zz4=)  Dark
Light
- Auto

 ![SVG Image](data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMjQgMjQiIGZpbGw9ImN1cnJlbnRDb2xvciIgc3R5bGU9Ii0tc2wtaWNvbi1zaXplOiAxZW07IiB3aWR0aD0iMTYiIGFyaWEtaGlkZGVuPSJ0cnVlIiBoZWlnaHQ9IjE2IiBjbGFzcz0iaWNvbiBjYXJldCBhc3Ryby00eXBodG9lbiBhc3Ryby1jNnZzb3FhcyI+PHBhdGggZD0iTTE3IDkuMTdhMSAxIDAgMCAwLTEuNDEgMEwxMiAxMi43MSA4LjQ2IDkuMTdhMSAxIDAgMSAwLTEuNDEgMS40Mmw0LjI0IDQuMjRhMS4wMDIgMS4wMDIgMCAwIDAgMS40MiAwTDE3IDEwLjU5YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)

**On this page![SVG Image](data:image/svg+xml;base64,PHN2ZyBzdHlsZT0iLS1zbC1pY29uLXNpemU6IDFyZW07IiB2aWV3Qm94PSIwIDAgMjQgMjQiIGZpbGw9ImN1cnJlbnRDb2xvciIgYXJpYS1oaWRkZW49InRydWUiIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiIgY2xhc3M9ImNhcmV0IGFzdHJvLWRveW5rNXRsIGFzdHJvLWM2dnNvcWFzIj48cGF0aCBkPSJtMTQuODMgMTEuMjktNC4yNC00LjI0YTEgMSAwIDEgMC0xLjQyIDEuNDFMMTIuNzEgMTJsLTMuNTQgMy41NGExIDEgMCAwIDAgMCAxLjQxIDEgMSAwIDAgMCAuNzEuMjkgMSAxIDAgMCAwIC43MS0uMjlsNC4yNC00LjI0YTEuMDAyIDEuMDAyIDAgMCAwIDAtMS40MloiIC8+PC9zdmc+)**

- [Introduction](#_top)
- [Install](#install)
- [Get started](#get-started)
- [Use Gemini CLI](#use-gemini-cli)
- [Features](#features)
- [Configuration](#configuration)
- [Reference](#reference)
- [Resources](#resources)
- [Development](#development)
- [Releases](#releases)

## On this page

- [Introduction](#_top)
- [Install](#install)
- [Get started](#get-started)
- [Use Gemini CLI](#use-gemini-cli)
- [Features](#features)
- [Configuration](#configuration)
- [Reference](#reference)
- [Resources](#resources)
- [Development](#development)
- [Releases](#releases)

# Gemini CLI documentation

  ![SVG Image](data:image/svg+xml;base64,PHN2ZyBzdHJva2U9ImN1cnJlbnRDb2xvciIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBoZWlnaHQ9IjE2IiBzdHJva2Utd2lkdGg9IjIiIHdpZHRoPSIxNiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBjbGFzcz0iYXN0cm8teW0yY2J5ZHkiPiA8cmVjdCByeD0iMiIgd2lkdGg9IjE0IiB4PSI4IiB5PSI4IiByeT0iMiIgaGVpZ2h0PSIxNCIgY2xhc3M9ImFzdHJvLXltMmNieWR5IiAvPiA8cGF0aCBkPSJtNCAxNmMtMS4xIDAtMi0uOS0yLTJWNGMwLTEuMS45LTIgMi0yaDEwYzEuMSAwIDIgLjkgMiAyIiBjbGFzcz0iYXN0cm8teW0yY2J5ZHkiIC8+IDwvc3ZnPg==) Copy as Markdown Copied!

Gemini CLI brings the power of Gemini models directly into your terminal. Use it
to understand code, automate tasks, and build workflows with your local project
context.

## Install

[![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Install”](#install)

 Terminal window

    npm install -g @google/gemini-cli

## Get started

[![SVG Image](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Get started”](#get-started)

Jump in to Gemini CLI.

- **[Quickstart](/docs/get-started):** Your first session with Gemini CLI.
- **[Installation](/docs/get-started/installation):** How to install Gemini CLI
on your system.
- **[Authentication](/docs/get-started/authentication):** Setup instructions for
personal and enterprise accounts.
- **[Examples](/docs/get-started/examples):** Practical examples of Gemini CLI in
action.
- **[CLI cheatsheet](/docs/cli/cli-reference):** A quick reference for common
commands and options.
- **[Gemini 3 on Gemini CLI](/docs/get-started/gemini-3):** Learn about Gemini 3
support in Gemini CLI.

## Use Gemini CLI

[![SVG Image](data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMjQgMjQiIGhlaWdodD0iMTYiIHdpZHRoPSIxNiI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Use Gemini CLI”](#use-gemini-cli)

User-focused guides and tutorials for daily development workflows.

- **[File management](/docs/cli/tutorials/file-management):** How to work with
local files and directories.
- **[Get started with Agent skills](/docs/cli/tutorials/skills-getting-started):** Getting started with specialized expertise.
- **[Manage context and memory](/docs/cli/tutorials/memory-management):** Managing persistent instructions and facts.
- **[Execute shell commands](/docs/cli/tutorials/shell-commands):** Executing
system commands safely.
- **[Manage sessions and history](/docs/cli/tutorials/session-management):** Resuming, managing, and rewinding conversations.
- **[Plan tasks with todos](/docs/cli/tutorials/task-planning):** Using todos for
complex workflows.
- **[Web search and fetch](/docs/cli/tutorials/web-tools):** Searching and
fetching content from the web.
- **[Set up an MCP server](/docs/cli/tutorials/mcp-setup):** Set up an MCP
server.
- **[Automate tasks](/docs/cli/tutorials/automation):** Automate tasks.

## Features

[![SVG Image](data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMjQgMjQiIGhlaWdodD0iMTYiIHdpZHRoPSIxNiI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Features”](#features)

Technical documentation for each capability of Gemini CLI.

- **[Extensions](/docs/extensions):** Extend Gemini CLI with new tools and
capabilities.
- **[Agent Skills](/docs/cli/skills):** Use specialized agents for specific
tasks.
- **[Checkpointing](/docs/cli/checkpointing):** Automatic session snapshots.
- **[Headless mode](/docs/cli/headless):** Programmatic and scripting interface.
- **[Hooks](/docs/hooks):** Customize Gemini CLI behavior with scripts.
- **[IDE integration](/docs/ide-integration):** Integrate Gemini CLI with
your favorite IDE.
- **[MCP servers](/docs/tools/mcp-server):** Connect to and use remote agents.
- **[Model routing](/docs/cli/model-routing):** Automatic fallback resilience.
- **[Model selection](/docs/cli/model):** Choose the best model for your needs.
- **[Plan mode 🔬](/docs/cli/plan-mode):** Use a safe, read-only mode for
planning complex changes.
- **[Subagents 🔬](/docs/core/subagents):** Using specialized agents for specific
tasks.
- **[Remote subagents 🔬](/docs/core/remote-agents):** Connecting to and using
remote agents.
- **[Rewind](/docs/cli/rewind):** Rewind and replay sessions.
- **[Sandboxing](/docs/cli/sandbox):** Isolate tool execution.
- **[Settings](/docs/cli/settings):** Full configuration reference.
- **[Telemetry](/docs/cli/telemetry):** Usage and performance metric details.
- **[Token caching](/docs/cli/token-caching):** Performance optimization.

## Configuration

[![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Configuration”](#configuration)

Settings and customization options for Gemini CLI.

- **[Custom commands](/docs/cli/custom-commands):** Personalized shortcuts.
- **[Enterprise configuration](/docs/cli/enterprise):** Professional environment
controls.
- **[Ignore files (.geminiignore)](/docs/cli/gemini-ignore):** Exclusion pattern
reference.
- **[Model configuration](/docs/cli/generation-settings):** Fine-tune generation
parameters like temperature and thinking budget.
- **[Project context (GEMINI.md)](/docs/cli/gemini-md):** Technical hierarchy of
context files.
- **[System prompt override](/docs/cli/system-prompt):** Instruction replacement
logic.
- **[Themes](/docs/cli/themes):** UI personalization technical guide.
- **[Trusted folders](/docs/cli/trusted-folders):** Security permission logic.

## Reference

[![SVG Image](data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Reference”](#reference)

Deep technical documentation and API specifications.

- **[Command reference](/docs/reference/commands):** Detailed slash command
guide.
- **[Configuration reference](/docs/reference/configuration):** Settings and
environment variables.
- **[Keyboard shortcuts](/docs/reference/keyboard-shortcuts):** Productivity
tips.
- **[Memory import processor](/docs/reference/memport):** How Gemini CLI
processes memory from various sources.
- **[Policy engine](/docs/reference/policy-engine):** Fine-grained execution
control.
- **[Tools reference](/docs/reference/tools):** Information on how tools are
defined, registered, and used.

## Resources

[![SVG Image](data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMjQgMjQiIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Resources”](#resources)

Support, release history, and legal information.

- **[FAQ](/docs/resources/faq):** Answers to frequently asked questions.
- **[Quota and pricing](/docs/resources/quota-and-pricing):** Limits and billing
details.
- **[Terms and privacy](/docs/resources/tos-privacy):** Official notices and
terms.
- **[Troubleshooting](/docs/resources/troubleshooting):** Common issues and
solutions.
- **[Uninstall](/docs/resources/uninstall):** How to uninstall Gemini CLI.

## Development

[![SVG Image](data:image/svg+xml;base64,PHN2ZyBoZWlnaHQ9IjE2IiB3aWR0aD0iMTYiIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Development”](#development)

- **[Contribution guide](/docs/contributing):** How to contribute to Gemini CLI.
- **[Integration testing](/docs/integration-tests):** Running integration tests.
- **[Issue and PR automation](/docs/issue-and-pr-automation):** Automation for
issues and pull requests.
- **[Local development](/docs/local-development):** Setting up a local
development environment.
- **[NPM package structure](/docs/npm):** The structure of the NPM packages.

## Releases

[![SVG Image](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTYiIHZpZXdCb3g9IjAgMCAyNCAyNCIgaGVpZ2h0PSIxNiI+PHBhdGggZmlsbD0iY3VycmVudGNvbG9yIiBkPSJtMTIuMTEgMTUuMzktMy44OCAzLjg4YTIuNTIgMi41MiAwIDAgMS0zLjUgMCAyLjQ3IDIuNDcgMCAwIDEgMC0zLjVsMy44OC0zLjg4YTEgMSAwIDAgMC0xLjQyLTEuNDJsLTMuODggMy44OWE0LjQ4IDQuNDggMCAwIDAgNi4zMyA2LjMzbDMuODktMy44OGExIDEgMCAxIDAtMS40Mi0xLjQyWm04LjU4LTEyLjA4YTQuNDkgNC40OSAwIDAgMC02LjMzIDBsLTMuODkgMy44OGExIDEgMCAwIDAgMS40MiAxLjQybDMuODgtMy44OGEyLjUyIDIuNTIgMCAwIDEgMy41IDAgMi40NyAyLjQ3IDAgMCAxIDAgMy41bC0zLjg4IDMuODhhMSAxIDAgMSAwIDEuNDIgMS40MmwzLjg4LTMuODlhNC40OSA0LjQ5IDAgMCAwIDAtNi4zM1pNOC44MyAxNS4xN2ExIDEgMCAwIDAgMS4xLjIyIDEgMSAwIDAgMCAuMzItLjIybDQuOTItNC45MmExIDEgMCAwIDAtMS40Mi0xLjQybC00LjkyIDQuOTJhMSAxIDAgMCAwIDAgMS40MloiIC8+PC9zdmc+)Section titled “Releases”](#releases)

- **[Release notes](/docs/changelogs):** Release notes for all versions.
- **[Stable release](/docs/changelogs/latest):** The latest stable release.
- **[Preview release](/docs/changelogs/preview):** The latest preview release.

This website uses [cookies](https://policies.google.com/technologies/cookies) from Google to deliver and enhance the quality of its services and to analyze
 traffic.

 I understand.

 ![Google logo](/assets/Google.svg) ![Google logo](/assets/Google-dark.svg) ![For Developers logo](/assets/fordevelopers.svg) ![For Developers logo](/assets/fordevelopers-dark.svg)

 [Terms](/terms) | [Privacy](https://policies.google.com/privacy)
