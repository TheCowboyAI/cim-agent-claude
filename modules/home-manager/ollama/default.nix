{ lib, config, ... }:
with lib;
let 
  cfg = config.ollama;
in {
  options.ollama = {
    enable = mkEnableOption "Enable Ollama service";
  };

  config = mkIf cfg.enable {
    services.ollama = {
      enable = true;
      acceleration = "cuda";
      openFirewall = true;
      loadModels = [
        "vicuna"
      ];
      host = "0.0.0.0";
      port = 11434;
      home = "/var/lib/ollama";
      environmentVariables = {
        OLLAMA_ORIGINS = "http://0.0.0.0:11434";
        OLLAMA_HOST = "0.0.0.0:11434";
        OLLAMA_MODELS = "/var/lib/ollama/models";
        OLLAMA_URL = "http://0.0.0.0:11434";
      };
    };
  };
}