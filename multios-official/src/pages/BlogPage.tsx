import React from 'react';

const BlogPage = () => {
  const featuredPost = {
    title: "MultiOS 1.0.0 Released: A Complete Educational Operating System",
    excerpt: "We're thrilled to announce the release of MultiOS 1.0.0, featuring comprehensive cross-platform support, modern Rust implementation, and extensive educational resources.",
    date: "November 3, 2025",
    author: "MultiOS Team",
    readTime: "8 min read",
    category: "Release",
    image: "/images/operating_system_architecture_cpu_memory_device_diagram_minimal_bw.jpg"
  };

  const blogPosts = [
    {
      title: "Building MultiOS: Architecture Decisions and Lessons Learned",
      excerpt: "A deep dive into the architectural decisions that shaped MultiOS, from the choice of Rust to cross-platform considerations.",
      date: "October 28, 2025",
      author: "Development Team",
      readTime: "12 min read",
      category: "Architecture"
    },
    {
      title: "Educational Impact: How MultiOS is Changing OS Education",
      excerpt: "Exploring how universities are integrating MultiOS into their operating systems curricula and the impact on student learning.",
      date: "October 21, 2025",
      author: "Education Team",
      readTime: "6 min read",
      category: "Education"
    },
    {
      title: "Performance Optimization: From 5 Seconds to 2 Seconds Boot Time",
      excerpt: "Technical details on how we optimized MultiOS boot performance through careful profiling and targeted optimizations.",
      date: "October 14, 2025",
      author: "Performance Team",
      readTime: "10 min read",
      category: "Performance"
    },
    {
      title: "Cross-Platform Development with Rust: Challenges and Solutions",
      excerpt: "Lessons learned from implementing MultiOS across x86_64, ARM64, and RISC-V architectures using Rust.",
      date: "October 7, 2025",
      author: "Platform Team",
      readTime: "15 min read",
      category: "Development"
    },
    {
      title: "Community Spotlight: Meet Our Contributors",
      excerpt: "Highlighting the amazing contributors who have made MultiOS possible, from students to industry professionals.",
      date: "September 30, 2025",
      author: "Community Team",
      readTime: "5 min read",
      category: "Community"
    },
    {
      title: "The Future of Operating Systems Education",
      excerpt: "Our vision for how MultiOS will transform operating systems education and research in the coming years.",
      date: "September 23, 2025",
      author: "Research Team",
      readTime: "8 min read",
      category: "Vision"
    }
  ];

  const categories = [
    { name: "All", count: 25 },
    { name: "Release", count: 8 },
    { name: "Architecture", count: 6 },
    { name: "Education", count: 4 },
    { name: "Performance", count: 3 },
    { name: "Development", count: 2 },
    { name: "Community", count: 2 }
  ];

  return (
    <div className="pt-16">
      {/* Page Header */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container text-center">
          <h1 className="text-h1 mb-6">Blog & News</h1>
          <p className="text-large max-w-3xl mx-auto">
            Stay updated with the latest MultiOS developments, technical deep-dives, educational insights, 
            and community highlights from the operating systems world.
          </p>
        </div>
      </section>

      {/* Featured Post */}
      <section className="section">
        <div className="container">
          <div className="max-w-5xl mx-auto">
            <div className="card-white">
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                <div>
                  <div className="mb-4">
                    <span className="inline-block px-3 py-1 text-xs font-bold uppercase tracking-wider bg-red-100 text-red-800">
                      Featured
                    </span>
                    <span className="ml-3 text-small text-gray-600">{featuredPost.category}</span>
                  </div>
                  <h2 className="text-h1 mb-4">{featuredPost.title}</h2>
                  <p className="text-large text-gray-600 mb-6">{featuredPost.excerpt}</p>
                  <div className="flex items-center text-small text-gray-600 mb-6">
                    <span>{featuredPost.author}</span>
                    <span className="mx-2">•</span>
                    <span>{featuredPost.date}</span>
                    <span className="mx-2">•</span>
                    <span>{featuredPost.readTime}</span>
                  </div>
                  <button className="btn btn-primary">
                    Read Full Article
                  </button>
                </div>
                <div>
                  <img 
                    src={featuredPost.image}
                    alt={featuredPost.title}
                    className="w-full h-64 object-cover border border-gray-light"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Main Content */}
      <section className="section" style={{ backgroundColor: 'var(--surface)' }}>
        <div className="container">
          <div className="max-w-6xl mx-auto">
            <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
              {/* Sidebar */}
              <div className="lg:col-span-1">
                <div className="card-white">
                  <h3 className="text-h3 mb-4">Categories</h3>
                  <ul className="space-y-2">
                    {categories.map((category, index) => (
                      <li key={index}>
                        <button className="flex justify-between w-full text-left py-2 text-body hover:text-red-600 transition-colors">
                          <span>{category.name}</span>
                          <span className="text-small text-gray-600">({category.count})</span>
                        </button>
                      </li>
                    ))}
                  </ul>
                </div>

                <div className="card-white mt-6">
                  <h3 className="text-h3 mb-4">Subscribe</h3>
                  <p className="text-body text-gray-600 mb-4">
                    Get notified about new posts and updates.
                  </p>
                  <div className="space-y-3">
                    <input 
                      type="email" 
                      placeholder="Your email"
                      className="w-full input"
                    />
                    <button className="w-full btn btn-primary">
                      Subscribe
                    </button>
                  </div>
                </div>

                <div className="card-white mt-6">
                  <h3 className="text-h3 mb-4">Archive</h3>
                  <ul className="space-y-2 text-small">
                    <li><a href="#" className="text-gray-600 hover:text-red-600">November 2025 (3)</a></li>
                    <li><a href="#" className="text-gray-600 hover:text-red-600">October 2025 (8)</a></li>
                    <li><a href="#" className="text-gray-600 hover:text-red-600">September 2025 (5)</a></li>
                    <li><a href="#" className="text-gray-600 hover:text-red-600">August 2025 (4)</a></li>
                    <li><a href="#" className="text-gray-600 hover:text-red-600">July 2025 (5)</a></li>
                  </ul>
                </div>
              </div>

              {/* Blog Posts Grid */}
              <div className="lg:col-span-3">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  {blogPosts.map((post, index) => (
                    <article key={index} className="card-white">
                      <div className="mb-4">
                        <span className="text-small text-gray-600">{post.category}</span>
                      </div>
                      <h3 className="text-h3 mb-3 line-clamp-2">{post.title}</h3>
                      <p className="text-body text-gray-600 mb-4 line-clamp-3">{post.excerpt}</p>
                      <div className="flex items-center text-small text-gray-600 mb-4">
                        <span>{post.author}</span>
                        <span className="mx-2">•</span>
                        <span>{post.date}</span>
                        <span className="mx-2">•</span>
                        <span>{post.readTime}</span>
                      </div>
                      <button className="text-small font-bold uppercase tracking-wider text-red-600 hover:text-red-700">
                        Read More →
                      </button>
                    </article>
                  ))}
                </div>

                {/* Load More */}
                <div className="text-center mt-8">
                  <button className="btn btn-secondary">
                    Load More Posts
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default BlogPage;