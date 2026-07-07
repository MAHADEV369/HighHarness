class Highharness < Formula
  desc "Governance for AI coding agents. Permissions, audit trails, tamper-proof hash chains."
  homepage "https://github.com/MAHADEV369/HighHarness"
  version "0.1.0"
  license "MIT"

  depends_on "rust" => :build

  # When tagging a release, update URL and SHA256:
  #   url "https://github.com/MAHADEV369/HighHarness/archive/refs/tags/v#{version}.tar.gz"
  #   sha256 "..."  # generate via: curl -sL <url> | sha256sum
  #
  # Until the first tagged release, install from crates.io:
  def install
    system "cargo", "install", "--root", prefix, "highharness"
  end

  test do
    assert_match "HighHarness", shell_output("#{bin}/HighHarness --version")
  end
end
