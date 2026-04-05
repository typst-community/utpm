class Utpm < Formula
  desc "Unofficial Typst Package Manager"
  homepage "https://github.com/typst-community/utpm"
  url "https://github.com/typst-community/utpm/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "daff23fe337df266426ed57a5b1cd9000cab72ec0d71c9eb117b4af23adfd138"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
    
    # Generate and install shell completions
    generate_completions_from_executable(bin/"utpm", "generate")
    
    # Install documentation
    doc.install "docs/GUIDE.md"
  end

  test do
    assert_match "utpm", shell_output("#{bin}/utpm --version")
  end
end
