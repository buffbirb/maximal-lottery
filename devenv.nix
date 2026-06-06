_: {
  # https://devenv.sh/integrations/codespaces-devcontainer/
  devcontainer = {
    enable = true;
  };

  # https://devenv.sh/git-hooks/
  git-hooks = {
    hooks = {
      actionlint = {
        enable = true;
      };
      clippy = {
        enable = true;
      };
      check-yaml = {
        enable = true;
      };
      deadnix = {
        enable = true;
      };
      nixfmt = {
        enable = true;
      };
      shellcheck = {
        enable = true;
      };
      statix = {
        enable = true;
      };
      rustfmt = {
        enable = true;
      };
    };
  };

  # https://devenv.sh/languages/
  languages = {
    rust = {
      enable = true;
    };
  };

  # See full reference at https://devenv.sh/reference/options/
}
