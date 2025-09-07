{ lib, config, pkgs, ... }:

with lib;
let
  lock-false = {
    Value = false;
    Status = "locked";
  };
  lock-true = {
    Value = true;
    Status = "locked";
  };
  cfg = config.firefox;
in
{
  options.firefox.enable = lib.mkEnableOption "Enable firefox";
  config = mkIf cfg.enable
    {
      programs.firefox = {
        enable = true;

        profiles = {
          default = {
            id = 0;
            isDefault = true;
          };
        };

        policies = {
          DisableTelemetry = true;
          DisableFirefoxStudies = true;
          EnableTrackingProtection = {
            Value = true;
            Locked = true;
            Cryptomining = true;
            Fingerprinting = true;
          };
          DisablePocket = true;
          DisableFirefoxAccounts = true;
          DisableAccounts = true;
          DisableFirefoxScreenshots = true;
          OverrideFirstRunPage = "";
          OverridePostUpdatePage = "";
          DontCheckDefaultBrowser = true;
          DisplayBookmarksToolbar = "never"; # alternatives: "always" or "newtab"
          DisplayMenuBar = "default-off"; # alternatives: "always", "never" or "default-on"
          SearchBar = "unified"; # alternative: "separate"

          ExtensionSettings = with builtins;
            let
              extension = shortId: uuid: {
                name = uuid;
                value = {
                  install_url = "https://addons.mozilla.org/en-US/firefox/downloads/latest/${shortId}/latest.xpi";
                  installation_mode = "normal_installed";
                };
              };
            in
            listToAttrs [
              (extension "tree-style-tab" "treestyletab@piro.sakura.ne.jp")
              (extension "ublock-origin" "uBlock0@raymondhill.net")
              (extension "bitwarden-password-manager" "{446900e4-71c2-419f-a6a7-df9c091e268b}")
              (extension "tabliss" "extension@tabliss.io")
              (extension "umatrix" "uMatrix@raymondhill.net")
              (extension "libredirect" "7esoorv3@alefvanoon.anonaddy.me")
              (extension "clearurls" "{74145f27-f039-47ce-a470-a662b129930a}")
            ];
          # To add additional extensions, find it on addons.mozilla.org, find
          # the short ID in the url (like https://addons.mozilla.org/en-US/firefox/addon/!SHORT_ID!/)
          # Then, download the XPI by filling it in to the install_url template, unzip it,
          # run `jq .browser_specific_settings.gecko.id manifest.json` or
          # `jq .applications.gecko.id manifest.json` to get the UUID
          Preferences = {
            settings = {
              "browser.send_pings" = false;
              "browser.urlbar.speculativeConnect.enabled" = false;
              "dom.event.clipboardevents.enabled" = true;
              "media.navigator.enabled" = false;
              "network.cookie.cookieBehavior" = 1;
              "network.http.referer.XOriginPolicy" = 2;
              "network.http.referer.XOriginTrimmingPolicy" = 2;
              "beacon.enabled" = false;
              "browser.safebrowsing.downloads.remote.enabled" = false;
              "network.IDN_show_punycode" = true;
              "extensions.activeThemeID" = "firefox-compact-dark@mozilla.org";
              "app.shield.optoutstudies.enabled" = false;
              "dom.security.https_only_mode_ever_enabled" = true;
              "toolkit.legacyUserProfileCustomizations.stylesheets" = true;
              "browser.toolbars.bookmarks.visibility" = "always";
              "geo.enabled" = false;

              # Disable telemetry
              "browser.newtabpage.activity-stream.feeds.telemetry" = false;
              "browser.ping-centre.telemetry" = false;
              "browser.tabs.crashReporting.sendReport" = false;
              "devtools.onboarding.telemetry.logged" = false;
              "toolkit.telemetry.enabled" = false;
              "toolkit.telemetry.unified" = false;
              "toolkit.telemetry.server" = "";

              # Disable Pocket
              "browser.newtabpage.activity-stream.feeds.discoverystreamfeed" = false;
              "browser.newtabpage.activity-stream.feeds.section.topstories" = false;
              "browser.newtabpage.activity-stream.section.highlights.includePocket" = false;
              "browser.newtabpage.activity-stream.showSponsored" = false;
              "extensions.pocket.enabled" = false;

              # Disable prefetching
              "network.dns.disablePrefetch" = true;
              "network.prefetch-next" = false;

              # Disable JS in PDFs
              "pdfjs.enableScripting" = false;

              # Harden SSL 
              "security.ssl.require_safe_negotiation" = true;

              # Extra
              "identity.fxaccounts.enabled" = false;
              "browser.search.suggest.enabled" = false;
              "browser.urlbar.shortcuts.bookmarks" = true;
              "browser.urlbar.shortcuts.history" = true;
              "browser.urlbar.shortcuts.tabs" = false;
              "browser.urlbar.suggest.bookmark" = false;
              "browser.urlbar.suggest.engines" = false;
              "browser.urlbar.suggest.history" = false;
              "browser.urlbar.suggest.openpage" = false;
              "browser.urlbar.suggest.topsites" = false;
              "browser.uidensity" = 1;
              "media.autoplay.enabled" = false;
              "toolkit.zoomManager.zoomValues" = ".8,.90,.95,1,1.1,1.2";

              "privacy.firstparty.isolate" = true;
              "network.http.sendRefererHeader" = 0;

              "dom.webgpu.enabled" = true;
            };
          };
        };
      };
    };
}
