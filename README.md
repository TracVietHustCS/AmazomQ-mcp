# PostgreSQL MCP Server for Amazon Q 

A Rust-based Modular Capability Provider (MCP) server that enables Amazon Q interact PostgreSQL database. Server allows users to query database struture read-only queries throught the Amazon Q chat Interface. 


## Overview 
This project implements a PostgreSQL server that follows the MCP protocol used by Amazon Q CLI
1. List all table in a PostgreSQL databases 
2. Get Schema info for specific tables
3. Execute read-only SQL queries with result return as JSON 
