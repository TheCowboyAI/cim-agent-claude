#!/usr/bin/env python3

import asyncio
import json
import os
import sys
import tempfile
import urllib.parse
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any, Dict, List, Optional

import aiohttp
import feedparser
import requests
import uvicorn
from bs4 import BeautifulSoup
from dateutil.parser import parse as parse_date
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

class ResearchMCPServer:
    def __init__(self, cache_path: str):
        self.cache_path = Path(cache_path)
        self.cache_path.mkdir(parents=True, exist_ok=True)
        
        self.available_tools = [
            {
                'name': 'arxiv_search',
                'description': 'Search ArXiv for research papers',
                'input_schema': {
                    'type': 'object',
                    'properties': {
                        'query': {
                            'type': 'string',
                            'description': 'Search query for papers'
                        },
                        'max_results': {
                            'type': 'integer',
                            'default': 10,
                            'description': 'Maximum number of results to return'
                        },
                        'sort_by': {
                            'type': 'string',
                            'enum': ['relevance', 'lastUpdatedDate', 'submittedDate'],
                            'default': 'relevance',
                            'description': 'Sort order for results'
                        }
                    },
                    'required': ['query']
                }
            },
            {
                'name': 'arxiv_get_paper',
                'description': 'Get detailed information about a specific ArXiv paper',
                'input_schema': {
                    'type': 'object',
                    'properties': {
                        'arxiv_id': {
                            'type': 'string',
                            'description': 'ArXiv ID (e.g., 2301.07041)'
                        },
                        'include_abstract': {
                            'type': 'boolean',
                            'default': True,
                            'description': 'Include paper abstract'
                        }
                    },
                    'required': ['arxiv_id']
                }
            },
            {
                'name': 'download_paper_pdf',
                'description': 'Download PDF of an ArXiv paper',
                'input_schema': {
                    'type': 'object',
                    'properties': {
                        'arxiv_id': {
                            'type': 'string',
                            'description': 'ArXiv ID (e.g., 2301.07041)'
                        }
                    },
                    'required': ['arxiv_id']
                }
            }
        ]
    
    async def handle_tool_call(self, tool_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        try:
            if tool_name == 'arxiv_search':
                return await self._arxiv_search(arguments)
            elif tool_name == 'arxiv_get_paper':
                return await self._arxiv_get_paper(arguments)
            elif tool_name == 'download_paper_pdf':
                return await self._download_paper_pdf(arguments)
            else:
                raise HTTPException(status_code=400, detail=f'Unknown tool: {tool_name}')
        except Exception as e:
            return {'error': str(e), 'success': False}
    
    async def _arxiv_search(self, args: Dict[str, Any]) -> Dict[str, Any]:
        query = args['query']
        max_results = args.get('max_results', 10)
        sort_by = args.get('sort_by', 'relevance')
        
        # Build ArXiv API URL
        base_url = 'http://export.arxiv.org/api/query'
        params = {
            'search_query': query,
            'max_results': max_results,
            'sortBy': sort_by
        }
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(base_url, params=params) as response:
                    content = await response.text()
                    
            # Parse the Atom feed
            feed = feedparser.parse(content)
            
            papers = []
            for entry in feed.entries:
                # Extract ArXiv ID from entry ID
                arxiv_id = entry.id.split('/')[-1]
                if 'v' in arxiv_id:
                    arxiv_id = arxiv_id.split('v')[0]
                
                paper = {
                    'arxiv_id': arxiv_id,
                    'title': entry.title,
                    'authors': [author.name for author in entry.authors] if hasattr(entry, 'authors') else [],
                    'abstract': entry.summary if hasattr(entry, 'summary') else '',
                    'published': entry.published if hasattr(entry, 'published') else '',
                    'updated': entry.updated if hasattr(entry, 'updated') else '',
                    'categories': [tag.term for tag in entry.tags] if hasattr(entry, 'tags') else [],
                    'pdf_url': f'https://arxiv.org/pdf/{arxiv_id}.pdf',
                    'abs_url': f'https://arxiv.org/abs/{arxiv_id}'
                }
                papers.append(paper)
            
            return {
                'papers': papers,
                'total_results': len(papers),
                'success': True
            }
            
        except Exception as e:
            return {'error': f'Failed to search ArXiv: {str(e)}', 'success': False}
    
    async def _arxiv_get_paper(self, args: Dict[str, Any]) -> Dict[str, Any]:
        arxiv_id = args['arxiv_id']
        include_abstract = args.get('include_abstract', True)
        
        # Search for the specific paper
        search_result = await self._arxiv_search({'query': f'id:{arxiv_id}', 'max_results': 1})
        
        if not search_result.get('success') or not search_result.get('papers'):
            return {'error': f'Paper {arxiv_id} not found', 'success': False}
        
        paper = search_result['papers'][0]
        
        if not include_abstract:
            paper.pop('abstract', None)
        
        return {
            'paper': paper,
            'success': True
        }
    
    async def _download_paper_pdf(self, args: Dict[str, Any]) -> Dict[str, Any]:
        arxiv_id = args['arxiv_id']
        
        # Create filename and check if already cached
        filename = f'{arxiv_id}.pdf'
        file_path = self.cache_path / filename
        
        if file_path.exists():
            return {
                'message': f'Paper {arxiv_id} already downloaded',
                'file_path': str(file_path),
                'success': True
            }
        
        # Download the PDF
        pdf_url = f'https://arxiv.org/pdf/{arxiv_id}.pdf'
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(pdf_url) as response:
                    if response.status == 200:
                        with open(file_path, 'wb') as f:
                            f.write(await response.read())
                        
                        return {
                            'message': f'Successfully downloaded paper {arxiv_id}',
                            'file_path': str(file_path),
                            'file_size': file_path.stat().st_size,
                            'success': True
                        }
                    else:
                        return {
                            'error': f'Failed to download paper {arxiv_id}: HTTP {response.status}',
                            'success': False
                        }
        
        except Exception as e:
            return {
                'error': f'Failed to download paper {arxiv_id}: {str(e)}',
                'success': False
            }


def create_app(cache_path: str) -> FastAPI:
    app = FastAPI(title='Research MCP Server', version='1.0.0')
    server = ResearchMCPServer(cache_path)
    
    @app.get('/mcp/tools')
    async def list_tools():
        return {'tools': server.available_tools}
    
    @app.post('/mcp/tools/{tool_name}')
    async def call_tool(tool_name: str, arguments: Dict[str, Any]):
        result = await server.handle_tool_call(tool_name, arguments)
        return {'result': result}
    
    @app.get('/health')
    async def health_check():
        return {'status': 'healthy', 'cache_path': cache_path}
    
    return app


if __name__ == '__main__':
    import argparse
    
    parser = argparse.ArgumentParser(description='Research MCP Server')
    parser.add_argument('--host', default='0.0.0.0', help='Host to bind to')
    parser.add_argument('--port', type=int, default=8005, help='Port to bind to')
    parser.add_argument('--cache-path', default='/var/lib/research-mcp/cache', help='Cache directory')
    parser.add_argument('--log-level', default='info', help='Log level')
    
    args = parser.parse_args()
    
    app = create_app(args.cache_path)
    uvicorn.run(app, host=args.host, port=args.port, log_level=args.log_level)