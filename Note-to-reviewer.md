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




```
