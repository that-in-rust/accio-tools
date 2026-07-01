# Note to Reviewer: 6 Hours Scope


``` text
This assignment has hit us on a rather busy week

I wish I had more time to do this but it is what it is

The original objective:
The task is to build a tool router: a component that, given a user query and the currently
connected tools, selects a small relevant subset before the agent reasons over it. You
will build the router, a minimal harness to exercise it, and an evaluation that measures
whether it works.

As a mathematical equation

- LHS
    - currently known variables
        - variable A: user query
        - variable B: currently connected tools related data including
            - name
            - description
            - parameter scheme
    - unknown variables
        - variable C: context of the user query from the orchestrator agent
- RHS
    - single tool call returned to the orchestrator agent
        - the tool call can be finalized basis the router we build


This is basically a search -> ranking -> selection problem 

When I started on this I got overwhelmed on the number of things I wanted to do on this so I said given the time contraints let me at least set a direction and call it phase 1 submission. If you need me to continue I will be happy to do a phase 2 submission with more features and better evaluation.

tl;dr

I imagine a multi-step workflow for us to arrive at the right tool

use variables A, B, C as inputs to

many approaches can work here


Approach 1:A two step bi-directional tool-selection workflow

a CPU tool (like BM25 search or similar algos) - shortlists top 5 tools

feeds into

an LLM judge (GPT 4.1 Mini API) which selections the top 1


Approach 2: A multi-step bidrectional tool-selection workflow

CPU tool shortlists top 10 tools
feeds into
a different CPU tool that further shortlists top 5 tools from the available 10 tools
feeds into
an LLM judge (GPT 4.1 Mini API) which selections the top 1

Do you see the pattern, I can take a number of combinations here, change the order and arrive at a different tool selection workflow

The meta pattern is:
Select a CPU tool to shortlist top M tools -> 
Select another CPU or LLM to shortlist N tools from the available M tools -> 
Select another CPU or LLM to shortlist O tools from the available N tools -> 

And each CPU tool here can be any search + ranking algo, RAG, actual existing tool infra like elasticsearch, vector search, etc. and each LLM can be any LLM that can do reasoning and selection


That is how expansive this problem statement is

So I have built a simple tool router with a UI so that I can actually test a query at a time and understand the different approaches and their results

This may not be exactly what you are looking for but this is the best I could do with the time in my hands

Hope this helps you understand the way I am thinking about this problem statement


```
