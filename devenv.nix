_: {
  # https://devenv.sh/git-hooks/
  git-hooks = {
    hooks = {
      actionlint = {
        enable = true;
      };
      check-yaml = {
        enable = true;
      };
      deadnix = {
        enable = true;
      };
      end-of-file-fixer = {
        enable = true;
        excludes = [
          "\\.lock$"
        ];
      };
      nixfmt = {
        enable = true;
      };
      rustfmt = {
        enable = true;
      };
      shellcheck = {
        enable = true;
      };
      statix = {
        enable = true;
      };
      taplo = {
        enable = true;
      };
      trim-trailing-whitespace = {
        enable = true;
        excludes = [
          "\\.lock$"
        ];
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
