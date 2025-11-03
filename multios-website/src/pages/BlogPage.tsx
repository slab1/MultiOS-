import React from 'react';
import { Calendar, User, ArrowRight, Clock } from 'lucide-react';

const BlogPage: React.FC = () => {
  const featuredPost = {
    title: 'MultiOS Beta Release: A Major Milestone in Educational Operating Systems',
    excerpt: 'We\'re excited to announce the beta release of MultiOS, marking a significant milestone in our journey to revolutionize operating systems education.',
    author: 'Dr. Sarah Chen',
    date: 'November 3, 2025',
    readTime: '8 min read',
    image: '/api/placeholder/600/300',
    category: 'Release News'
  };

  const blogPosts = [
    {
      title: 'Cross-Architecture Development: Lessons from MultiOS',
      excerpt: 'Learn how we achieved seamless multi-architecture support and the challenges we faced along the way.',
      author: 'Prof. Michael Rodriguez',
      date: 'October 28, 2025',
      readTime: '6 min read',
      category: 'Technical Deep Dive'
    },
    {
      title: 'Teaching Operating Systems in 2025: Modern Approaches',
      excerpt: 'How MultiOS is changing the way universities teach operating systems concepts.',
      author: 'Emily Watson',
      date: 'October 25, 2025',
      readTime: '5 min read',
      category: 'Education'
    },
    {
      title: 'Memory Safety in Operating Systems: A Rust Perspective',
      excerpt: 'Exploring how Rust\'s memory safety guarantees benefit operating systems development.',
      author: 'Dr. James Liu',
      date: 'October 20, 2025',
      readTime: '10 min read',
      category: 'Research'
    },
    {
      title: 'Building a Community: The MultiOS Story',
      excerpt: 'How we built a thriving community of developers, students, and educators around our project.',
      author: 'Community Team',
      date: 'October 15, 2025',
      readTime: '7 min read',
      category: 'Community'
    },
    {
      title: 'Performance Optimization Techniques for Educational OS',
      excerpt: 'Strategies for maintaining educational clarity while achieving production-level performance.',
      author: 'Alex Thompson',
      date: 'October 10, 2025',
      readTime: '8 min read',
      category: 'Performance'
    },
    {
      title: 'The Future of Operating Systems Education',
      excerpt: 'Our vision for the future of teaching and learning operating systems concepts.',
      author: 'Dr. Sarah Chen',
      date: 'October 5, 2025',
      readTime: '12 min read',
      category: 'Vision'
    }
  ];

  const categories = [
    'All Posts',
    'Release News',
    'Technical Deep Dive',
    'Education',
    'Research',
    'Community',
    'Performance',
    'Vision'
  ];

  return (
    <div className="min-h-screen bg-white">
      {/* Hero Section */}
      <section className="bg-gradient-to-br from-blue-600 via-purple-600 to-indigo-800 py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-white">
          <h1 className="text-4xl md:text-5xl font-bold mb-6">MultiOS Blog</h1>
          <p className="text-xl md:text-2xl text-blue-100 max-w-3xl mx-auto">
            Latest news, technical insights, and educational resources from the MultiOS community
          </p>
        </div>
      </section>

      {/* Featured Post */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-gray-900 mb-8">Featured Post</h2>
          <div className="bg-white rounded-xl shadow-lg overflow-hidden border border-gray-200">
            <div className="md:flex">
              <div className="md:w-1/2">
                <img 
                  src={featuredPost.image} 
                  alt={featuredPost.title}
                  className="w-full h-64 md:h-full object-cover"
                />
              </div>
              <div className="md:w-1/2 p-8">
                <span className="inline-block bg-blue-100 text-blue-800 text-sm px-3 py-1 rounded-full mb-4">
                  {featuredPost.category}
                </span>
                <h3 className="text-2xl font-bold text-gray-900 mb-4">{featuredPost.title}</h3>
                <p className="text-gray-600 mb-6">{featuredPost.excerpt}</p>
                <div className="flex items-center text-sm text-gray-500 mb-6">
                  <User className="w-4 h-4 mr-1" />
                  <span className="mr-4">{featuredPost.author}</span>
                  <Calendar className="w-4 h-4 mr-1" />
                  <span className="mr-4">{featuredPost.date}</span>
                  <Clock className="w-4 h-4 mr-1" />
                  <span>{featuredPost.readTime}</span>
                </div>
                <a
                  href="#"
                  className="inline-flex items-center text-blue-600 hover:text-blue-700 font-medium"
                >
                  Read Full Article
                  <ArrowRight className="w-4 h-4 ml-1" />
                </a>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Category Filter */}
      <section className="py-8 bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-wrap gap-2">
            {categories.map((category, index) => (
              <button
                key={index}
                className={`px-4 py-2 rounded-full text-sm font-medium transition-colors ${
                  index === 0
                    ? 'bg-blue-600 text-white'
                    : 'bg-white text-gray-700 hover:bg-gray-100 border border-gray-300'
                }`}
              >
                {category}
              </button>
            ))}
          </div>
        </div>
      </section>

      {/* Blog Posts Grid */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-2xl font-bold text-gray-900 mb-8">Latest Posts</h2>
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {blogPosts.map((post, index) => (
              <article key={index} className="bg-white rounded-xl shadow-lg overflow-hidden border border-gray-200 hover:shadow-xl transition-shadow">
                <div className="h-48 bg-gray-200"></div>
                <div className="p-6">
                  <span className="inline-block bg-gray-100 text-gray-800 text-sm px-3 py-1 rounded-full mb-3">
                    {post.category}
                  </span>
                  <h3 className="text-xl font-semibold text-gray-900 mb-3">{post.title}</h3>
                  <p className="text-gray-600 mb-4">{post.excerpt}</p>
                  <div className="flex items-center justify-between text-sm text-gray-500">
                    <div className="flex items-center">
                      <User className="w-4 h-4 mr-1" />
                      <span>{post.author}</span>
                    </div>
                    <div className="flex items-center">
                      <Calendar className="w-4 h-4 mr-1" />
                      <span>{post.date}</span>
                    </div>
                  </div>
                  <div className="mt-4 flex items-center justify-between">
                    <div className="flex items-center text-sm text-gray-500">
                      <Clock className="w-4 h-4 mr-1" />
                      <span>{post.readTime}</span>
                    </div>
                    <a
                      href="#"
                      className="inline-flex items-center text-blue-600 hover:text-blue-700 font-medium"
                    >
                      Read More
                      <ArrowRight className="w-4 h-4 ml-1" />
                    </a>
                  </div>
                </div>
              </article>
            ))}
          </div>

          {/* Load More Button */}
          <div className="text-center mt-12">
            <button className="px-8 py-3 bg-gray-100 text-gray-700 font-semibold rounded-lg hover:bg-gray-200 transition-colors">
              Load More Posts
            </button>
          </div>
        </div>
      </section>

      {/* Newsletter Signup */}
      <section className="py-20 bg-gray-50">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 className="text-3xl font-bold text-gray-900 mb-4">Stay Updated</h2>
          <p className="text-xl text-gray-600 mb-8">
            Subscribe to our newsletter for the latest updates, tutorials, and community news
          </p>
          <div className="flex max-w-md mx-auto">
            <input
              type="email"
              placeholder="Enter your email"
              className="flex-1 px-4 py-3 border border-gray-300 rounded-l-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button className="px-6 py-3 bg-blue-600 text-white font-semibold rounded-r-lg hover:bg-blue-700 transition-colors">
              Subscribe
            </button>
          </div>
        </div>
      </section>
    </div>
  );
};

export default BlogPage;