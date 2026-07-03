class Highharness < Formula
  desc "Runtime-neutral agent governance for AI coding agents"
  homepage "https://github.com/MAHADEV369/HighHarness"
  url "https://github.com/MAHADEV369/HighHarness/archive/refs/tags/v0.1.0.tar.gz"
  sha256 ""
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    assert_match "HighHarness", shell_output("#{bin}/HighHarness --version")
  end
end
