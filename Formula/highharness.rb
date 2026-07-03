class Highharness < Formula
  desc "Governance for AI coding agents. Permissions, audit trails, tamper-proof hash chains."
  homepage "https://github.com/MAHADEV369/HighHarness"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "highharness"
  end

  test do
    assert_match "HighHarness", shell_output("#{bin}/HighHarness --version")
  end
end
