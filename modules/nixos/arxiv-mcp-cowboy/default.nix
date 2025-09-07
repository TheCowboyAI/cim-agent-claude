{ config, lib, pkgs, ... }:

let
  
  # Create a custom fastmcp package without runtime dependency checks
  fastmcp-fixed = pkgs.python312Packages.fastmcp.overridePythonAttrs (old: {
    dontCheckRuntimeDeps = true;
    pythonImportsCheck = [ ];
    doCheck = false;
  });

  # Create arxiv package for dependencies
  arxiv-pkg = pkgs.python312Packages.buildPythonPackage rec {
    pname = "arxiv";
    version = "2.1.3";
    format = "setuptools";
    src = pkgs.fetchPypi {
      inherit pname version;
      hash = "sha256-MjZSIZlNLPBWV8H632OibvyMzewYWQKB7gNRW/74vE4=";
    };
    propagatedBuildInputs = with pkgs.python312Packages; [ requests feedparser ];
    doCheck = false;
  };

  # Build the arxiv-mcp-server package properly using buildPythonApplication
  arxiv-mcp-cowboy-pkg = pkgs.python312Packages.buildPythonApplication rec {
    pname = "arxiv-mcp-cowboy";
    version = "1.0.0";
    format = "pyproject";

    src = pkgs.fetchFromGitHub {
      owner = "TheCowboyAI";
      repo = "arxiv-mcp-server";
      rev = "057e2000be7b56823239815b0fe7c7fc0dbced96";
      hash = "sha256-R7guwxeQBQViZR/qD0AUhsQiL8XptLs1SkPZESd8ETc=";
    };

    nativeBuildInputs = with pkgs.python312Packages; [
      hatchling
    ];

    propagatedBuildInputs = with pkgs.python312Packages; [
      fastmcp-fixed
      arxiv-pkg
      requests
      python-dateutil
      httpx
      pydantic
      typing-extensions
      aiofiles
      aiohttp
      python-dotenv
      pydantic-settings
      uvicorn
      sse-starlette
      anyio
      black
      # Add pymupdf4llm - might need to define it
      (pkgs.python312Packages.buildPythonPackage rec {
        pname = "pymupdf4llm";
        version = "0.0.17";
        format = "setuptools";
        src = pkgs.fetchPypi {
          inherit pname version;
          hash = "sha256-Jyh++f4CF883hBo+8rz3DaJVPEPZXqObZkpt5khWeMM=";
        };
        propagatedBuildInputs = with pkgs.python312Packages; [ pymupdf markdown ];
        doCheck = false;
      })
    ];

    # Disable checks since they may require network access
    doCheck = false;
    pythonImportsCheck = [ ];
    dontCheckRuntimeDeps = true;

    meta = with lib; {
      description = "TheCowboyAI ArXiv MCP Server";
      homepage = "https://github.com/TheCowboyAI/arxiv-mcp-server";
      license = licenses.mit;
      maintainers = [ ];
      mainProgram = "arxiv-mcp-server";
    };
  };


in
{
  # No options needed - just install the binary

  config = {
    # Simply install the binary - no systemd service
    environment.systemPackages = [ arxiv-mcp-cowboy-pkg ];
    
    # Create storage directory with proper permissions
    systemd.tmpfiles.rules = [
      "d /var/lib/arxiv-mcp/papers 0755 root root -"
    ];
  };
}