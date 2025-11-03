const crypto = require('crypto');
const querystring = require('querystring');

class LTIProviderService {
  constructor(config) {
    this.consumerKey = config.consumerKey || process.env.LTI_KEY;
    this.consumerSecret = config.consumerSecret || process.env.LTI_SECRET;
    this.loginUrl = config.loginUrl || process.env.LTI_LOGIN_URL;
    this.launchUrl = config.launchUrl || process.env.LTI_LAUNCH_URL;
    this.launchPresentation = config.launchPresentation || {};
    this.customParameters = config.customParameters || {};
  }

  // Generate LTI launch parameters
  generateLaunchParams(resourceLink, userId, roles, contextId, toolProviderInfo = {}) {
    const timestamp = Math.floor(Date.now() / 1000);
    
    const baseParams = {
      lti_version: 'LTI-1p0',
      lti_message_type: 'basic-lti-launch-request',
      resource_link_id: resourceLink.id || 'default',
      resource_link_title: resourceLink.title || 'MultiOS Learning Tool',
      resource_link_description: resourceLink.description || '',
      
      user_id: userId,
      lis_person_name_given: toolProviderInfo.user_given_name || '',
      lis_person_name_family: toolProviderInfo.user_family_name || '',
      lis_person_contact_email_primary: toolProviderInfo.user_email || '',
      
      roles: roles.join(','),
      
      context_id: contextId,
      context_title: toolProviderInfo.context_title || 'Course',
      context_label: toolProviderInfo.context_label || 'Course',
      
      tool_consumer_instance_guid: this.consumerKey,
      tool_consumer_instance_name: 'MultiOS Course Platform',
      tool_consumer_instance_description: 'MultiOS Operating System Learning Platform',
      tool_consumer_instance_url: 'https://multios.org',
      tool_consumer_instance_contact_email: 'support@multios.org',
      
      launch_presentation_return_url: this.launchPresentation.return_url || `${this.launchUrl}?return=true`,
      launch_presentation_document_target: this.launchPresentation.document_target || 'iframe',
      launch_presentation_width: this.launchPresentation.width || '100%',
      launch_presentation_height: this.launchPresentation.height || '600',
      launch_presentation_locale: this.launchPresentation.locale || 'en',
      
      oauth_callback: 'about:blank',
      oauth_consumer_key: this.consumerKey,
      oauth_nonce: crypto.randomBytes(16).toString('hex'),
      oauth_signature_method: 'HMAC-SHA1',
      oauth_timestamp: timestamp.toString(),
      oauth_version: '1.0'
    };

    // Add custom parameters
    Object.keys(this.customParameters).forEach(key => {
      baseParams[`custom_${key}`] = this.customParameters[key];
    });

    return baseParams;
  }

  // Sign LTI parameters with OAuth
  signParameters(params, method = 'POST', url) {
    const baseParams = { ...params };
    
    // Create base string for signature
    const normalizedParams = this.normalizeParameters(baseParams);
    const baseString = this.createBaseString(method.toUpperCase(), url, normalizedParams);
    
    // Generate signature
    const signingKey = `${encodeURIComponent(this.consumerSecret)}&`;
    const signature = crypto
      .createHmac('sha1', signingKey)
      .update(baseString)
      .digest('base64');
    
    baseParams.oauth_signature = signature;
    
    return baseParams;
  }

  // Normalize parameters for OAuth signature
  normalizeParameters(params) {
    const normalized = {};
    
    // Extract and sort parameters
    Object.keys(params)
      .filter(key => key !== 'oauth_signature')
      .sort()
      .forEach(key => {
        normalized[key] = params[key];
      });
    
    return normalized;
  }

  // Create OAuth base string
  createBaseString(method, url, params) {
    const encodedUrl = encodeURIComponent(url);
    const encodedParams = encodeURIComponent(querystring.stringify(params));
    
    return `${method.toUpperCase()}&${encodedUrl}&${encodedParams}`;
  }

  // Validate LTI launch request
  validateLaunchRequest(params) {
    try {
      // Verify OAuth signature
      if (!this.verifyOAuthSignature(params)) {
        throw new Error('Invalid OAuth signature');
      }

      // Validate required parameters
      const requiredParams = [
        'lti_version',
        'lti_message_type',
        'resource_link_id',
        'user_id',
        'oauth_consumer_key',
        'oauth_signature_method',
        'oauth_signature',
        'oauth_timestamp',
        'oauth_nonce'
      ];

      for (const param of requiredParams) {
        if (!params[param]) {
          throw new Error(`Missing required parameter: ${param}`);
        }
      }

      // Validate LTI version
      if (params.lti_version !== 'LTI-1p0') {
        throw new Error('Unsupported LTI version');
      }

      // Validate message type
      if (params.lti_message_type !== 'basic-lti-launch-request') {
        throw new Error('Invalid LTI message type');
      }

      // Validate timestamp (within 5 minutes)
      const timestamp = parseInt(params.oauth_timestamp);
      const now = Math.floor(Date.now() / 1000);
      if (Math.abs(now - timestamp) > 300) {
        throw new Error('Request timestamp too old');
      }

      // Validate consumer key
      if (params.oauth_consumer_key !== this.consumerKey) {
        throw new Error('Invalid consumer key');
      }

      return {
        valid: true,
        userId: params.user_id,
        roles: params.roles ? params.roles.split(',') : [],
        contextId: params.context_id,
        resourceLinkId: params.resource_link_id,
        contextTitle: params.context_title,
        userInfo: {
          givenName: params.lis_person_name_given,
          familyName: params.lis_person_name_family,
          email: params.lis_person_contact_email_primary
        }
      };
    } catch (error) {
      return {
        valid: false,
        error: error.message
      };
    }
  }

  // Verify OAuth signature
  verifyOAuthSignature(params) {
    const providedSignature = params.oauth_signature;
    const method = params.method || 'POST';
    const url = params.launch_url || this.launchUrl;
    
    // Create signature without the signature parameter
    const paramsWithoutSignature = { ...params };
    delete paramsWithoutSignature.oauth_signature;
    delete paramsWithoutSignature.method;
    delete paramsWithoutSignature.launch_url;
    
    const expectedSignature = this.signParameters(paramsWithoutSignature, method, url).oauth_signature;
    
    return providedSignature === expectedSignature;
  }

  // Generate LTI configuration XML
  generateConfigXML(toolProviderConfig = {}) {
    const config = {
      title: toolProviderConfig.title || 'MultiOS Learning Platform',
      description: toolProviderConfig.description || 'Comprehensive operating system learning platform',
      launch_url: this.launchUrl,
      secure_launch_url: this.launchUrl.replace('http://', 'https://'),
      icon: toolProviderConfig.icon || `${this.launchUrl}/icon.png`,
      secure_icon: toolProviderConfig.secure_icon || `${this.launchUrl}/icon.png`,
      cartridge_bundle: 'BLTI001_Icon_1',
      cartridge_icon: 'BLTI001_Icon_1.png',
      ...toolProviderConfig
    };

    let xml = '<?xml version="1.0" encoding="UTF-8"?>' +
              '<cartridge_basiclti_link xmlns="http://www.imsglobal.org/xsd/imslticc_v1p0"' +
              '    xmlns:blti="http://www.imsglobal.org/xsd/imsbasiclti_v1p0"' +
              '    xmlns:lticm="http://www.imsglobal.org/xsd/imslticm_v1p0"' +
              '    xmlns:lticp="http://www.imsglobal.org/xsd/imslticp_v1p0"' +
              '    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"' +
              '    xsi:schemaLocation="http://www.imsglobal.org/xsd/imslticc_v1p0 ' +
              '    http://www.imsglobal.org/xsd/imslticc_v1p0.xsd ' +
              '    http://www.imsglobal.org/xsd/imsbasiclti_v1p0 ' +
              '    http://www.imsglobal.org/xsd/imsbasiclti_v1p0.xsd ' +
              '    http://www.imsglobal.org/xsd/imslticm_v1p0 ' +
              '    http://www.imsglobal.org/xsd/imslticm_v1p0.xsd ' +
              '    http://www.imsglobal.org/xsd/imslticp_v1p0 ' +
              '    http://www.imsglobal.org/xsd/imslticp_v1p0.xsd">' +
              `<blti:title>${config.title}</blti:title>` +
              `<blti:description>${config.description}</blti:description>` +
              `<blti:launch_url>${config.launch_url}</blti:launch_url>` +
              `<blti:secure_launch_url>${config.secure_launch_url}</blti:secure_launch_url>` +
              `<blti:icon>${config.icon}</blti:icon>` +
              `<blti:secure_icon>${config.secure_icon}</blti:secure_icon>` +
              `<blti:vendor>` +
              `<lticp:code>multios</lticp:code>` +
              `<lticp:name>MultiOS</lticp:name>` +
              `<lticp:description>MultiOS Development Team</lticp:description>` +
              `<lticp:url>https://multios.org</lticp:url>` +
              `<lticp:contact>` +
              `<lticp:email>support@multios.org</lticp:email>` +
              `</lticp:contact>` +
              `</blti:vendor>` +
              `<cartridge_bundle identifierref="BLTI001_Icon_1"/>` +
              `<cartridge_icon identifierref="BLTI001_Icon_1.png"/>`;

    // Add custom parameters
    if (config.custom_parameters) {
      Object.keys(config.custom_parameters).forEach(key => {
        xml += `<lticm:property name="custom_${key}">${config.custom_parameters[key]}</lticm:property>`;
      });
    }

    xml += '</cartridge_basiclti_link>';
    return xml;
  }

  // Handle LTI login request
  handleLoginRequest(params) {
    try {
      const requiredParams = ['lti_message_type', 'lti_version', 'resource_link_id'];
      
      for (const param of requiredParams) {
        if (!params[param]) {
          throw new Error(`Missing required parameter: ${param}`);
        }
      }

      if (params.lti_message_type !== 'LTIResourceLinkRequest') {
        throw new Error('Invalid LTI message type for login');
      }

      const returnUrl = params.launch_presentation_return_url || this.launchUrl;
      
      // Redirect to login handler
      return {
        redirectUrl: `${this.loginUrl}?${querystring.stringify({
          lti_message_type: params.lti_message_type,
          lti_version: params.lti_version,
          resource_link_id: params.resource_link_id,
          user_id: params.user_id,
          roles: params.roles,
          context_id: params.context_id,
          return_url: returnUrl
        })}`
      };
    } catch (error) {
      throw new Error(`LTI login error: ${error.message}`);
    }
  }

  // Handle LTI launch request
  handleLaunchRequest(params) {
    const validation = this.validateLaunchRequest(params);
    
    if (!validation.valid) {
      throw new Error(`LTI launch validation failed: ${validation.error}`);
    }

    return {
      success: true,
      launchData: validation,
      redirectUrl: this.launchPresentation.target || this.launchUrl
    };
  }

  // Generate outcome service parameters
  generateOutcomeParams(consumerKey, userId, resultSourcedId, score) {
    const timestamp = Math.floor(Date.now() / 1000);
    
    return {
      lti_version: 'LTI-1p0',
      lti_message_type: 'basic-lti-launch-request',
      user_id: userId,
      roles: 'Learner',
      resource_link_id: 'outcome_service',
      lis_result_sourcedid: resultSourcedId,
      lis_outcome_service_url: `${this.launchUrl}/outcomes`,
      ext_ims_lis_basic_outcome_url: `${this.launchUrl}/outcomes`,
      ext_ims_lis_resultvalue_sourcedids: 'decimal',
      oauth_consumer_key: consumerKey || this.consumerKey,
      oauth_nonce: crypto.randomBytes(16).toString('hex'),
      oauth_signature_method: 'HMAC-SHA1',
      oauth_timestamp: timestamp.toString(),
      oauth_version: '1.0',
      score: score
    };
  }

  // Send outcome result
  async sendOutcome(outcomeParams, outcomeUrl) {
    try {
      const signedParams = this.signParameters(outcomeParams, 'POST', outcomeUrl);
      
      const response = await fetch(outcomeUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
          'Authorization': this.buildOAuthHeader(signedParams, 'POST', outcomeUrl)
        },
        body: querystring.stringify(signedParams)
      });

      if (!response.ok) {
        throw new Error(`Outcome service error: ${response.statusText}`);
      }

      const result = await response.text();
      return { success: true, result };
    } catch (error) {
      return { success: false, error: error.message };
    }
  }

  // Build OAuth header
  buildOAuthHeader(params, method, url) {
    const oauthParams = {};
    
    Object.keys(params)
      .filter(key => key.startsWith('oauth_'))
      .sort()
      .forEach(key => {
        oauthParams[key] = params[key];
      });

    const normalizedParams = this.normalizeParameters(oauthParams);
    const baseString = this.createBaseString(method, url, normalizedParams);
    const signingKey = `${encodeURIComponent(this.consumerSecret)}&`;
    const signature = crypto
      .createHmac('sha1', signingKey)
      .update(baseString)
      .digest('base64');

    oauthParams.oauth_signature = signature;
    
    const authParams = Object.keys(oauthParams)
      .map(key => `${key}="${encodeURIComponent(oauthParams[key])}"`)
      .join(', ');
    
    return `OAuth ${authParams}`;
  }

  // Process LTI deep linking request
  handleDeepLinkingRequest(params) {
    if (params.lti_message_type !== 'LtiDeepLinkingRequest') {
      throw new Error('Invalid message type for deep linking');
    }

    // Generate content item selection
    const contentItems = [
      {
        type: 'ltiResourceLink',
        title: 'MultiOS Learning Module',
        text: 'Interactive MultiOS operating system learning content',
        url: this.launchUrl,
        custom: {
          course_id: params.custom_course_id || 'default',
          module_type: 'interactive'
        }
      }
    ];

    return {
      deep_link_return_url: params.deep_link_return_url,
      data: params.data,
      content_items: contentItems
    };
  }

  // Generate deep linking response
  generateDeepLinkingResponse(deepLinkData) {
    const response = {
      lti_message_type: 'LtiDeepLinkingResponse',
      lti_version: 'LTI-1p0',
      deployment_id: deepLinkData.deployment_id,
      data: deepLinkData.data,
      content_items: deepLinkData.content_items
    };

    return this.signParameters(response, 'POST', deepLinkData.deep_link_return_url);
  }

  // LTI Configuration management
  saveToolConfiguration(config) {
    // In a real implementation, save to database
    const configs = this.getToolConfigurations();
    const existingIndex = configs.findIndex(c => c.id === config.id);
    
    if (existingIndex >= 0) {
      configs[existingIndex] = { ...configs[existingIndex], ...config, updated_at: new Date() };
    } else {
      configs.push({ ...config, id: crypto.randomUUID(), created_at: new Date() });
    }
    
    return configs;
  }

  getToolConfigurations() {
    // In a real implementation, fetch from database
    return [];
  }

  // Webhook handling for LTI events
  handleWebhook(eventType, data) {
    switch (eventType) {
      case 'launch':
        return this.handleLaunchRequest(data);
      
      case 'login':
        return this.handleLoginRequest(data);
      
      case 'deep_linking':
        return this.handleDeepLinkingRequest(data);
      
      default:
        throw new Error(`Unknown LTI event type: ${eventType}`);
    }
  }

  // Disconnect/cleanup
  async disconnect() {
    // No specific cleanup needed for LTI provider
    return true;
  }
}

module.exports = LTIProviderService;