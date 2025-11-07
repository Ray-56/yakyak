use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Billing account status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Suspended,
    Overdue,
    Closed,
}

/// Currency code (ISO 4217)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CNY,
    Custom(String),
}

impl Currency {
    pub fn symbol(&self) -> &str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::GBP => "£",
            Currency::JPY => "¥",
            Currency::CNY => "¥",
            Currency::Custom(_) => "",
        }
    }
}

/// Billing cycle frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillingCycle {
    Monthly,
    Quarterly,
    Yearly,
    PayAsYouGo,
}

/// Usage type for billing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UsageType {
    InboundMinutes,
    OutboundMinutes,
    InternalMinutes,
    TollFreeMinutes,
    InternationalMinutes,
    SmsOutbound,
    SmsInbound,
    StorageGB,
    Recording,
    Conference,
    Custom(String),
}

/// Rate for a specific usage type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rate {
    pub usage_type: UsageType,
    pub rate_per_unit: f64,
    pub minimum_charge: f64,
    pub included_units: f64,
}

impl Rate {
    pub fn new(usage_type: UsageType, rate_per_unit: f64) -> Self {
        Self {
            usage_type,
            rate_per_unit,
            minimum_charge: 0.0,
            included_units: 0.0,
        }
    }

    pub fn with_minimum(mut self, minimum_charge: f64) -> Self {
        self.minimum_charge = minimum_charge;
        self
    }

    pub fn with_included_units(mut self, included_units: f64) -> Self {
        self.included_units = included_units;
        self
    }

    pub fn calculate_charge(&self, units: f64) -> f64 {
        if units <= self.included_units {
            self.minimum_charge
        } else {
            let billable_units = units - self.included_units;
            let charge = billable_units * self.rate_per_unit;
            charge.max(self.minimum_charge)
        }
    }
}

/// Rate plan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatePlan {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub currency: Currency,
    pub billing_cycle: BillingCycle,
    pub monthly_fee: f64,
    pub rates: Vec<Rate>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

impl RatePlan {
    pub fn new(name: String, currency: Currency, billing_cycle: BillingCycle) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: String::new(),
            currency,
            billing_cycle,
            monthly_fee: 0.0,
            rates: Vec::new(),
            active: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_monthly_fee(mut self, fee: f64) -> Self {
        self.monthly_fee = fee;
        self
    }

    pub fn add_rate(mut self, rate: Rate) -> Self {
        self.rates.push(rate);
        self
    }

    pub fn get_rate(&self, usage_type: &UsageType) -> Option<&Rate> {
        self.rates.iter().find(|r| &r.usage_type == usage_type)
    }

    pub fn calculate_usage_charge(&self, usage_type: &UsageType, units: f64) -> f64 {
        if let Some(rate) = self.get_rate(usage_type) {
            rate.calculate_charge(units)
        } else {
            0.0
        }
    }
}

/// Usage record for billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub usage_type: UsageType,
    pub quantity: f64,
    pub rate: f64,
    pub amount: f64,
    pub description: String,
    pub reference_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl UsageRecord {
    pub fn new(
        account_id: Uuid,
        usage_type: UsageType,
        quantity: f64,
        rate: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            usage_type,
            quantity,
            rate,
            amount: quantity * rate,
            description: String::new(),
            reference_id: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_reference(mut self, reference_id: String) -> Self {
        self.reference_id = Some(reference_id);
        self
    }
}

/// Invoice status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Draft,
    Issued,
    Paid,
    Overdue,
    Cancelled,
    Refunded,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub amount: f64,
}

impl InvoiceLineItem {
    pub fn new(description: String, quantity: f64, unit_price: f64) -> Self {
        Self {
            description,
            quantity,
            unit_price,
            amount: quantity * unit_price,
        }
    }
}

/// Invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub invoice_number: String,
    pub account_id: Uuid,
    pub status: InvoiceStatus,
    pub currency: Currency,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub line_items: Vec<InvoiceLineItem>,
    pub subtotal: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub amount_paid: f64,
    pub issued_at: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Invoice {
    pub fn new(
        account_id: Uuid,
        invoice_number: String,
        currency: Currency,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            invoice_number,
            account_id,
            status: InvoiceStatus::Draft,
            currency,
            period_start,
            period_end,
            line_items: Vec::new(),
            subtotal: 0.0,
            tax_rate: 0.0,
            tax_amount: 0.0,
            total: 0.0,
            amount_paid: 0.0,
            issued_at: None,
            due_at: None,
            paid_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn add_line_item(&mut self, item: InvoiceLineItem) {
        self.line_items.push(item);
        self.recalculate_total();
    }

    pub fn set_tax_rate(&mut self, tax_rate: f64) {
        self.tax_rate = tax_rate;
        self.recalculate_total();
    }

    fn recalculate_total(&mut self) {
        self.subtotal = self.line_items.iter().map(|item| item.amount).sum();
        self.tax_amount = self.subtotal * self.tax_rate;
        self.total = self.subtotal + self.tax_amount;
    }

    pub fn issue(&mut self, due_days: u32) {
        self.status = InvoiceStatus::Issued;
        let now = Utc::now();
        self.issued_at = Some(now);
        self.due_at = Some(now + chrono::Duration::days(due_days as i64));
    }

    pub fn mark_paid(&mut self, amount: f64) {
        self.amount_paid += amount;
        if self.amount_paid >= self.total {
            self.status = InvoiceStatus::Paid;
            self.paid_at = Some(Utc::now());
        }
    }

    pub fn mark_overdue(&mut self) {
        if self.status == InvoiceStatus::Issued {
            if let Some(due_at) = self.due_at {
                if Utc::now() > due_at {
                    self.status = InvoiceStatus::Overdue;
                }
            }
        }
    }

    pub fn is_overdue(&self) -> bool {
        self.status == InvoiceStatus::Overdue
            || (self.status == InvoiceStatus::Issued
                && self.due_at.map_or(false, |due| Utc::now() > due))
    }

    pub fn balance_due(&self) -> f64 {
        (self.total - self.amount_paid).max(0.0)
    }
}

/// Payment method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    CreditCard { last4: String, brand: String },
    BankTransfer { account_number: String },
    PayPal { email: String },
    Other(String),
}

/// Payment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub account_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub amount: f64,
    pub currency: Currency,
    pub method: PaymentMethod,
    pub reference: String,
    pub notes: String,
    pub processed_at: DateTime<Utc>,
}

impl Payment {
    pub fn new(
        account_id: Uuid,
        amount: f64,
        currency: Currency,
        method: PaymentMethod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            invoice_id: None,
            amount,
            currency,
            method,
            reference: String::new(),
            notes: String::new(),
            processed_at: Utc::now(),
        }
    }

    pub fn for_invoice(mut self, invoice_id: Uuid) -> Self {
        self.invoice_id = Some(invoice_id);
        self
    }

    pub fn with_reference(mut self, reference: String) -> Self {
        self.reference = reference;
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = notes;
        self
    }
}

/// Billing account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAccount {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: AccountStatus,
    pub rate_plan_id: Uuid,
    pub currency: Currency,
    pub balance: f64,
    pub credit_limit: f64,
    pub auto_pay: bool,
    pub billing_contact_email: String,
    pub billing_contact_name: String,
    pub billing_address: String,
    pub tax_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_invoice_date: Option<DateTime<Utc>>,
}

impl BillingAccount {
    pub fn new(
        tenant_id: Uuid,
        rate_plan_id: Uuid,
        currency: Currency,
        billing_contact_email: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            status: AccountStatus::Active,
            rate_plan_id,
            currency,
            balance: 0.0,
            credit_limit: 0.0,
            auto_pay: false,
            billing_contact_email,
            billing_contact_name: String::new(),
            billing_address: String::new(),
            tax_id: None,
            created_at: Utc::now(),
            last_invoice_date: None,
        }
    }

    pub fn set_credit_limit(&mut self, limit: f64) {
        self.credit_limit = limit;
    }

    pub fn enable_auto_pay(&mut self) {
        self.auto_pay = true;
    }

    pub fn add_charge(&mut self, amount: f64) {
        self.balance += amount;
        if self.balance > self.credit_limit && self.credit_limit > 0.0 {
            self.status = AccountStatus::Suspended;
        }
    }

    pub fn add_payment(&mut self, amount: f64) {
        self.balance -= amount;
        if self.balance <= self.credit_limit {
            if self.status == AccountStatus::Suspended {
                self.status = AccountStatus::Active;
            }
        }
    }

    pub fn is_suspended(&self) -> bool {
        self.status == AccountStatus::Suspended
    }

    pub fn is_over_limit(&self) -> bool {
        self.credit_limit > 0.0 && self.balance > self.credit_limit
    }
}

/// Billing manager for handling all billing operations
pub struct BillingManager {
    accounts: Arc<Mutex<HashMap<Uuid, BillingAccount>>>,
    rate_plans: Arc<Mutex<HashMap<Uuid, RatePlan>>>,
    usage_records: Arc<Mutex<Vec<UsageRecord>>>,
    invoices: Arc<Mutex<HashMap<Uuid, Invoice>>>,
    payments: Arc<Mutex<Vec<Payment>>>,
    next_invoice_number: Arc<Mutex<u64>>,
}

impl BillingManager {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(Mutex::new(HashMap::new())),
            rate_plans: Arc::new(Mutex::new(HashMap::new())),
            usage_records: Arc::new(Mutex::new(Vec::new())),
            invoices: Arc::new(Mutex::new(HashMap::new())),
            payments: Arc::new(Mutex::new(Vec::new())),
            next_invoice_number: Arc::new(Mutex::new(1)),
        }
    }

    /// Create a new billing account
    pub fn create_account(&self, account: BillingAccount) -> Uuid {
        let account_id = account.id;
        self.accounts.lock().unwrap().insert(account_id, account);
        account_id
    }

    /// Get a billing account
    pub fn get_account(&self, account_id: &Uuid) -> Option<BillingAccount> {
        self.accounts.lock().unwrap().get(account_id).cloned()
    }

    /// Update account status
    pub fn update_account_status(&self, account_id: &Uuid, status: AccountStatus) -> bool {
        if let Some(account) = self.accounts.lock().unwrap().get_mut(account_id) {
            account.status = status;
            true
        } else {
            false
        }
    }

    /// Create a rate plan
    pub fn create_rate_plan(&self, plan: RatePlan) -> Uuid {
        let plan_id = plan.id;
        self.rate_plans.lock().unwrap().insert(plan_id, plan);
        plan_id
    }

    /// Get a rate plan
    pub fn get_rate_plan(&self, plan_id: &Uuid) -> Option<RatePlan> {
        self.rate_plans.lock().unwrap().get(plan_id).cloned()
    }

    /// Record usage
    pub fn record_usage(
        &self,
        account_id: Uuid,
        usage_type: UsageType,
        quantity: f64,
    ) -> Result<Uuid, String> {
        // Get account and rate plan
        let account = self
            .get_account(&account_id)
            .ok_or("Account not found")?;

        if account.is_suspended() {
            return Err("Account is suspended".to_string());
        }

        let rate_plan = self
            .get_rate_plan(&account.rate_plan_id)
            .ok_or("Rate plan not found")?;

        // Calculate charge
        let rate = rate_plan
            .get_rate(&usage_type)
            .map(|r| r.rate_per_unit)
            .unwrap_or(0.0);

        let record = UsageRecord::new(account_id, usage_type, quantity, rate);
        let record_id = record.id;

        // Add charge to account
        self.accounts
            .lock()
            .unwrap()
            .get_mut(&account_id)
            .unwrap()
            .add_charge(record.amount);

        // Store usage record
        self.usage_records.lock().unwrap().push(record);

        Ok(record_id)
    }

    /// Generate invoice for an account
    pub fn generate_invoice(
        &self,
        account_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Uuid, String> {
        let account = self
            .get_account(&account_id)
            .ok_or("Account not found")?;

        let rate_plan = self
            .get_rate_plan(&account.rate_plan_id)
            .ok_or("Rate plan not found")?;

        // Generate invoice number
        let invoice_number = {
            let mut counter = self.next_invoice_number.lock().unwrap();
            let num = format!("INV-{:08}", *counter);
            *counter += 1;
            num
        };

        let mut invoice = Invoice::new(
            account_id,
            invoice_number,
            account.currency.clone(),
            period_start,
            period_end,
        );

        // Add monthly fee
        if rate_plan.monthly_fee > 0.0 {
            invoice.add_line_item(InvoiceLineItem::new(
                format!("{} - Monthly Fee", rate_plan.name),
                1.0,
                rate_plan.monthly_fee,
            ));
        }

        // Aggregate usage by type
        let usage_records = self.usage_records.lock().unwrap();
        let mut usage_aggregates: HashMap<String, (f64, f64)> = HashMap::new();

        for record in usage_records.iter() {
            if record.account_id == account_id
                && record.timestamp >= period_start
                && record.timestamp <= period_end
            {
                let key = format!("{:?}", record.usage_type);
                let entry = usage_aggregates.entry(key.clone()).or_insert((0.0, 0.0));
                entry.0 += record.quantity;
                entry.1 += record.amount;
            }
        }

        // Add usage line items
        for (usage_type, (quantity, amount)) in usage_aggregates {
            invoice.add_line_item(InvoiceLineItem::new(
                usage_type,
                quantity,
                amount / quantity,
            ));
        }

        let invoice_id = invoice.id;
        self.invoices.lock().unwrap().insert(invoice_id, invoice);

        // Update account
        self.accounts
            .lock()
            .unwrap()
            .get_mut(&account_id)
            .unwrap()
            .last_invoice_date = Some(Utc::now());

        Ok(invoice_id)
    }

    /// Issue an invoice
    pub fn issue_invoice(&self, invoice_id: &Uuid, due_days: u32) -> Result<(), String> {
        if let Some(invoice) = self.invoices.lock().unwrap().get_mut(invoice_id) {
            invoice.issue(due_days);
            Ok(())
        } else {
            Err("Invoice not found".to_string())
        }
    }

    /// Record a payment
    pub fn record_payment(&self, payment: Payment) -> Result<Uuid, String> {
        let payment_id = payment.id;

        // Apply payment to account
        if let Some(account) = self.accounts.lock().unwrap().get_mut(&payment.account_id) {
            account.add_payment(payment.amount);
        } else {
            return Err("Account not found".to_string());
        }

        // If payment is for an invoice, update invoice
        if let Some(invoice_id) = payment.invoice_id {
            if let Some(invoice) = self.invoices.lock().unwrap().get_mut(&invoice_id) {
                invoice.mark_paid(payment.amount);
            }
        }

        self.payments.lock().unwrap().push(payment);
        Ok(payment_id)
    }

    /// Get invoices for an account
    pub fn get_account_invoices(&self, account_id: &Uuid) -> Vec<Invoice> {
        self.invoices
            .lock()
            .unwrap()
            .values()
            .filter(|inv| &inv.account_id == account_id)
            .cloned()
            .collect()
    }

    /// Mark overdue invoices
    pub fn mark_overdue_invoices(&self) {
        for invoice in self.invoices.lock().unwrap().values_mut() {
            invoice.mark_overdue();

            // Update account status if invoice is overdue
            if invoice.is_overdue() {
                self.update_account_status(&invoice.account_id, AccountStatus::Overdue);
            }
        }
    }

    /// Get account balance
    pub fn get_account_balance(&self, account_id: &Uuid) -> Option<f64> {
        self.accounts
            .lock()
            .unwrap()
            .get(account_id)
            .map(|acc| acc.balance)
    }

    /// Get usage summary for period
    pub fn get_usage_summary(
        &self,
        account_id: &Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> HashMap<UsageType, (f64, f64)> {
        let mut summary = HashMap::new();
        let usage_records = self.usage_records.lock().unwrap();

        for record in usage_records.iter() {
            if &record.account_id == account_id
                && record.timestamp >= start
                && record.timestamp <= end
            {
                let entry = summary
                    .entry(record.usage_type.clone())
                    .or_insert((0.0, 0.0));
                entry.0 += record.quantity;
                entry.1 += record.amount;
            }
        }

        summary
    }
}

impl Default for BillingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_calculation() {
        let rate = Rate::new(UsageType::OutboundMinutes, 0.05);
        assert_eq!(rate.calculate_charge(100.0), 5.0);

        let rate_with_included = Rate::new(UsageType::InboundMinutes, 0.03)
            .with_included_units(50.0);
        assert_eq!(rate_with_included.calculate_charge(30.0), 0.0);
        assert_eq!(rate_with_included.calculate_charge(100.0), 1.5);
    }

    #[test]
    fn test_rate_plan() {
        let plan = RatePlan::new(
            "Standard Plan".to_string(),
            Currency::USD,
            BillingCycle::Monthly,
        )
        .with_monthly_fee(29.99)
        .add_rate(Rate::new(UsageType::OutboundMinutes, 0.05))
        .add_rate(Rate::new(UsageType::InboundMinutes, 0.02));

        assert_eq!(plan.monthly_fee, 29.99);
        assert_eq!(plan.rates.len(), 2);
        assert_eq!(
            plan.calculate_usage_charge(&UsageType::OutboundMinutes, 100.0),
            5.0
        );
    }

    #[test]
    fn test_invoice_lifecycle() {
        let mut invoice = Invoice::new(
            Uuid::new_v4(),
            "INV-001".to_string(),
            Currency::USD,
            Utc::now(),
            Utc::now(),
        );

        invoice.add_line_item(InvoiceLineItem::new(
            "Monthly Fee".to_string(),
            1.0,
            29.99,
        ));
        invoice.add_line_item(InvoiceLineItem::new(
            "Outbound Minutes".to_string(),
            100.0,
            0.05,
        ));

        assert_eq!(invoice.subtotal, 34.99);
        invoice.set_tax_rate(0.10);
        assert!((invoice.total - 38.489).abs() < 0.01);

        invoice.issue(30);
        assert_eq!(invoice.status, InvoiceStatus::Issued);
        assert!(invoice.issued_at.is_some());

        invoice.mark_paid(invoice.total);
        assert_eq!(invoice.status, InvoiceStatus::Paid);
        assert_eq!(invoice.balance_due(), 0.0);
    }

    #[test]
    fn test_billing_account() {
        let mut account = BillingAccount::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Currency::USD,
            "billing@example.com".to_string(),
        );

        account.set_credit_limit(100.0);
        assert_eq!(account.balance, 0.0);

        account.add_charge(50.0);
        assert_eq!(account.balance, 50.0);
        assert!(!account.is_suspended());

        account.add_charge(60.0);
        assert_eq!(account.balance, 110.0);
        assert!(account.is_suspended());

        account.add_payment(20.0);
        assert_eq!(account.balance, 90.0);
        assert!(!account.is_suspended());
    }

    #[test]
    fn test_billing_manager_usage() {
        let manager = BillingManager::new();

        // Create rate plan
        let plan = RatePlan::new(
            "Test Plan".to_string(),
            Currency::USD,
            BillingCycle::Monthly,
        )
        .add_rate(Rate::new(UsageType::OutboundMinutes, 0.10));
        let plan_id = manager.create_rate_plan(plan);

        // Create account
        let account = BillingAccount::new(
            Uuid::new_v4(),
            plan_id,
            Currency::USD,
            "test@example.com".to_string(),
        );
        let account_id = manager.create_account(account);

        // Record usage
        let result = manager.record_usage(account_id, UsageType::OutboundMinutes, 100.0);
        assert!(result.is_ok());

        // Check balance
        let balance = manager.get_account_balance(&account_id);
        assert_eq!(balance, Some(10.0));
    }

    #[test]
    fn test_invoice_generation() {
        let manager = BillingManager::new();

        // Create rate plan
        let plan = RatePlan::new(
            "Premium Plan".to_string(),
            Currency::USD,
            BillingCycle::Monthly,
        )
        .with_monthly_fee(49.99)
        .add_rate(Rate::new(UsageType::OutboundMinutes, 0.08));
        let plan_id = manager.create_rate_plan(plan);

        // Create account
        let account = BillingAccount::new(
            Uuid::new_v4(),
            plan_id,
            Currency::USD,
            "premium@example.com".to_string(),
        );
        let account_id = manager.create_account(account);

        // Record usage
        manager
            .record_usage(account_id, UsageType::OutboundMinutes, 200.0)
            .unwrap();

        // Generate invoice
        let start = Utc::now() - chrono::Duration::days(30);
        let end = Utc::now();
        let invoice_id = manager.generate_invoice(account_id, start, end).unwrap();

        let invoices = manager.get_account_invoices(&account_id);
        assert_eq!(invoices.len(), 1);
        assert!(invoices[0].subtotal > 49.99);
    }

    #[test]
    fn test_payment_processing() {
        let manager = BillingManager::new();

        // Setup account
        let plan = RatePlan::new("Test".to_string(), Currency::USD, BillingCycle::Monthly);
        let plan_id = manager.create_rate_plan(plan);

        let mut account = BillingAccount::new(
            Uuid::new_v4(),
            plan_id,
            Currency::USD,
            "test@example.com".to_string(),
        );
        account.add_charge(100.0);
        let account_id = manager.create_account(account);

        // Process payment
        let payment = Payment::new(
            account_id,
            50.0,
            Currency::USD,
            PaymentMethod::CreditCard {
                last4: "4242".to_string(),
                brand: "Visa".to_string(),
            },
        );

        let result = manager.record_payment(payment);
        assert!(result.is_ok());

        let balance = manager.get_account_balance(&account_id);
        assert_eq!(balance, Some(50.0));
    }

    #[test]
    fn test_usage_summary() {
        let manager = BillingManager::new();

        let plan = RatePlan::new("Test".to_string(), Currency::USD, BillingCycle::Monthly)
            .add_rate(Rate::new(UsageType::OutboundMinutes, 0.05))
            .add_rate(Rate::new(UsageType::InboundMinutes, 0.02));
        let plan_id = manager.create_rate_plan(plan);

        let account = BillingAccount::new(
            Uuid::new_v4(),
            plan_id,
            Currency::USD,
            "test@example.com".to_string(),
        );
        let account_id = manager.create_account(account);

        // Record multiple usage
        manager
            .record_usage(account_id, UsageType::OutboundMinutes, 100.0)
            .unwrap();
        manager
            .record_usage(account_id, UsageType::OutboundMinutes, 50.0)
            .unwrap();
        manager
            .record_usage(account_id, UsageType::InboundMinutes, 200.0)
            .unwrap();

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let summary = manager.get_usage_summary(&account_id, start, end);

        assert_eq!(summary.len(), 2);
        assert_eq!(summary.get(&UsageType::OutboundMinutes).unwrap().0, 150.0);
        assert_eq!(summary.get(&UsageType::InboundMinutes).unwrap().0, 200.0);
    }
}
